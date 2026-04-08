use std::{
    collections::HashSet,
    path::Path,
    path::PathBuf,
    sync::{Mutex, OnceLock},
    time::Duration,
};

use anyhow::{anyhow, Context, Result};
use chrono::Utc;
use keyring::Entry;
use rusqlite::{params, Connection, OptionalExtension};
use uuid::Uuid;

use crate::db::{
    models::{
        GithubAccountDTO, GithubExternalItemDTO, GithubSettingsDTO, GithubStatusDTO,
        GithubSyncResultDTO, RepoDTO, RepoSubDTO,
    },
    repo,
};

use super::{
    client::GithubClient,
    models::{ExternalCandidate, NotificationItem},
    rules,
};

static SYNC_IN_PROGRESS: OnceLock<Mutex<HashSet<String>>> = OnceLock::new();
static POLLER_STARTED: OnceLock<()> = OnceLock::new();

const KEYRING_SERVICE: &str = "kitodo";

#[derive(Debug, Clone)]
pub struct GithubSettingsPatch {
    pub enabled: Option<bool>,
    pub sync_interval_sec: Option<i64>,
    pub import_pr_reviews: Option<bool>,
    pub import_assigned_issues: Option<bool>,
    pub import_notifications: Option<bool>,
    pub default_project_id: Option<Option<String>>,
}

#[derive(Debug, Clone)]
struct GithubSettingsRow {
    enabled: bool,
    sync_interval_sec: i64,
    import_pr_reviews: bool,
    import_assigned_issues: bool,
    import_notifications: bool,
    default_project_id: Option<String>,
}

pub fn spawn_poller(db_path: PathBuf) {
    if POLLER_STARTED.set(()).is_err() {
        return;
    }

    std::thread::spawn(move || loop {
        if let Err(e) = run_poll_tick(&db_path) {
            eprintln!("[github-sync] poll tick error: {e}");
        }
        std::thread::sleep(Duration::from_secs(30));
    });
}

pub fn github_connect(db_path: &Path, token: &str) -> Result<GithubAccountDTO> {
    repo::init_db(db_path)?;
    let token = token.trim();
    if token.is_empty() {
        return Err(anyhow!("Token vacío"));
    }

    let client = GithubClient::new(token.to_string())?;
    let user = client.get_user().context("Token inválido: /user falló")?;
    let token_kind = client
        .detect_token_kind()
        .unwrap_or_else(|_| "unknown".to_string());

    let now = now_iso();
    let account_id = uuid_v7();
    let conn = open_conn(db_path)?;

    let existing = conn
        .query_row(
            "SELECT account_id, created_at FROM github_accounts WHERE username = ?1",
            params![user.login],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
        )
        .optional()?;

    let (account_id, created_at) = if let Some((existing_id, existing_created_at)) = existing {
        conn.execute(
            "UPDATE github_accounts SET token_kind = ?2, updated_at = ?3 WHERE account_id = ?1",
            params![existing_id, token_kind, now],
        )?;
        (existing_id, existing_created_at)
    } else {
        conn.execute(
            "INSERT INTO github_accounts(account_id, username, created_at, updated_at, token_kind) VALUES(?1, ?2, ?3, ?3, ?4)",
            params![account_id, user.login, now, token_kind],
        )?;
        conn.execute(
            "INSERT INTO github_settings(account_id) VALUES(?1)",
            params![account_id],
        )?;
        (account_id, now.clone())
    };

    set_token(&account_id, token)?;

    Ok(GithubAccountDTO {
        account_id,
        username: user.login,
        token_kind,
        created_at,
    })
}

pub fn github_disconnect(db_path: &Path, account_id: &str) -> Result<bool> {
    repo::init_db(db_path)?;
    let conn = open_conn(db_path)?;
    conn.execute(
        "UPDATE github_settings SET enabled = 0 WHERE account_id = ?1",
        params![account_id],
    )?;

    clear_token(account_id)?;
    Ok(true)
}

pub fn github_set_settings(
    db_path: &Path,
    account_id: &str,
    patch: GithubSettingsPatch,
) -> Result<GithubSettingsDTO> {
    repo::init_db(db_path)?;
    let conn = open_conn(db_path)?;

    let mut current = get_settings_row(&conn, account_id)?;

    if let Some(v) = patch.enabled {
        current.enabled = v;
    }
    if let Some(v) = patch.sync_interval_sec {
        current.sync_interval_sec = v.clamp(60, 3600);
    }
    if let Some(v) = patch.import_pr_reviews {
        current.import_pr_reviews = v;
    }
    if let Some(v) = patch.import_assigned_issues {
        current.import_assigned_issues = v;
    }
    if let Some(v) = patch.import_notifications {
        current.import_notifications = v;
    }
    if let Some(v) = patch.default_project_id {
        current.default_project_id = v;
    }

    conn.execute(
        "
        UPDATE github_settings
        SET enabled = ?2,
            sync_interval_sec = ?3,
            import_pr_reviews = ?4,
            import_assigned_issues = ?5,
            import_notifications = ?6,
            default_project_id = ?7
        WHERE account_id = ?1
        ",
        params![
            account_id,
            bool_to_i64(current.enabled),
            current.sync_interval_sec,
            bool_to_i64(current.import_pr_reviews),
            bool_to_i64(current.import_assigned_issues),
            bool_to_i64(current.import_notifications),
            current.default_project_id,
        ],
    )?;

    get_settings_dto(&conn, account_id)
}

pub fn github_get_settings(db_path: &Path, account_id: &str) -> Result<GithubSettingsDTO> {
    repo::init_db(db_path)?;
    let conn = open_conn(db_path)?;
    get_settings_dto(&conn, account_id)
}

pub fn github_list_accounts(db_path: &Path) -> Result<Vec<GithubAccountDTO>> {
    repo::init_db(db_path)?;
    let conn = open_conn(db_path)?;

    let mut stmt = conn.prepare(
        "SELECT account_id, username, token_kind, created_at FROM github_accounts ORDER BY created_at DESC",
    )?;

    let rows = stmt.query_map([], |row| {
        Ok(GithubAccountDTO {
            account_id: row.get(0)?,
            username: row.get(1)?,
            token_kind: row.get(2)?,
            created_at: row.get(3)?,
        })
    })?;

    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

pub fn github_list_repos(db_path: &Path, account_id: &str) -> Result<Vec<RepoDTO>> {
    repo::init_db(db_path)?;
    let token = get_token(account_id)?;
    let client = GithubClient::new(token)?;

    let repos = client.list_repos()?;
    Ok(repos
        .into_iter()
        .map(|r| RepoDTO {
            owner: r.owner.login,
            repo: r.name,
            full_name: r.full_name,
            private: r.private,
        })
        .collect())
}

pub fn github_add_repo_subscription(
    db_path: &Path,
    account_id: &str,
    owner: &str,
    repo_name: &str,
) -> Result<RepoSubDTO> {
    repo::init_db(db_path)?;
    let conn = open_conn(db_path)?;

    let id = uuid_v7();
    conn.execute(
        "
        INSERT INTO github_repo_subscriptions(id, account_id, owner, repo, enabled)
        VALUES(?1, ?2, ?3, ?4, 1)
        ON CONFLICT(account_id, owner, repo) DO UPDATE SET enabled = 1
        ",
        params![id, account_id, owner.trim(), repo_name.trim()],
    )?;

    let sub_id = conn.query_row(
        "SELECT id FROM github_repo_subscriptions WHERE account_id = ?1 AND owner = ?2 AND repo = ?3",
        params![account_id, owner.trim(), repo_name.trim()],
        |row| row.get::<_, String>(0),
    )?;

    get_repo_sub_by_id(&conn, &sub_id)
}

pub fn github_remove_repo_subscription(db_path: &Path, id: &str) -> Result<bool> {
    repo::init_db(db_path)?;
    let conn = open_conn(db_path)?;
    let changed = conn.execute(
        "DELETE FROM github_repo_subscriptions WHERE id = ?1",
        params![id],
    )?;
    Ok(changed > 0)
}

pub fn github_toggle_repo_subscription(
    db_path: &Path,
    id: &str,
    enabled: bool,
) -> Result<RepoSubDTO> {
    repo::init_db(db_path)?;
    let conn = open_conn(db_path)?;
    conn.execute(
        "UPDATE github_repo_subscriptions SET enabled = ?2 WHERE id = ?1",
        params![id, bool_to_i64(enabled)],
    )?;
    get_repo_sub_by_id(&conn, id)
}

pub fn github_list_repo_subscriptions(db_path: &Path, account_id: &str) -> Result<Vec<RepoSubDTO>> {
    repo::init_db(db_path)?;
    let conn = open_conn(db_path)?;

    let mut stmt = conn.prepare(
        "
        SELECT id, account_id, owner, repo, enabled, last_synced_at
        FROM github_repo_subscriptions
        WHERE account_id = ?1
        ORDER BY owner COLLATE NOCASE, repo COLLATE NOCASE
        ",
    )?;

    let rows = stmt.query_map(params![account_id], map_repo_sub)?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

pub fn github_sync_now(db_path: &Path, account_id: &str) -> Result<GithubSyncResultDTO> {
    sync_for_account(db_path, account_id)
}

pub fn github_get_status(db_path: &Path, account_id: &str) -> Result<GithubStatusDTO> {
    repo::init_db(db_path)?;
    let conn = open_conn(db_path)?;

    let (username, enabled): (String, i64) = conn
        .query_row(
            "
            SELECT a.username, s.enabled
            FROM github_accounts a
            JOIN github_settings s ON s.account_id = a.account_id
            WHERE a.account_id = ?1
            ",
            params![account_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()?
        .ok_or_else(|| anyhow!("Cuenta GitHub no encontrada"))?;

    let last_sync_at: Option<String> = conn.query_row(
        "SELECT MAX(last_synced_at) FROM github_repo_subscriptions WHERE account_id = ?1",
        params![account_id],
        |row| row.get(0),
    )?;

    Ok(GithubStatusDTO {
        account_id: account_id.to_string(),
        username,
        enabled: enabled == 1,
        last_sync_at,
        last_error: None,
    })
}

pub fn github_list_external_items(
    db_path: &Path,
    limit: i64,
) -> Result<Vec<GithubExternalItemDTO>> {
    repo::init_db(db_path)?;
    let conn = open_conn(db_path)?;

    let mut stmt = conn.prepare(
        "
        SELECT id, kind, url, title, state, repo_full, number, updated_at_ext
        FROM external_items
        WHERE source = 'github'
        ORDER BY COALESCE(updated_at_ext, updated_at) DESC
        LIMIT ?1
        ",
    )?;

    let rows = stmt.query_map(params![limit.clamp(1, 500)], |row| {
        Ok(GithubExternalItemDTO {
            id: row.get(0)?,
            kind: row.get(1)?,
            url: row.get(2)?,
            title: row.get(3)?,
            state: row.get(4)?,
            repo_full: row.get(5)?,
            number: row.get(6)?,
            updated_at_ext: row.get(7)?,
        })
    })?;

    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

fn run_poll_tick(db_path: &Path) -> Result<()> {
    repo::init_db(db_path)?;
    let conn = open_conn(db_path)?;

    let mut stmt = conn.prepare(
        "
        SELECT a.account_id, COALESCE(s.sync_interval_sec, 300), MAX(rs.last_synced_at)
        FROM github_accounts a
        JOIN github_settings s ON s.account_id = a.account_id
        LEFT JOIN github_repo_subscriptions rs ON rs.account_id = a.account_id AND rs.enabled = 1
        WHERE s.enabled = 1
        GROUP BY a.account_id, s.sync_interval_sec
        ",
    )?;

    let rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, i64>(1)?,
            row.get::<_, Option<String>>(2)?,
        ))
    })?;

    for row in rows {
        let (account_id, interval, last_synced_opt) = row?;
        let should_sync = match last_synced_opt {
            None => true,
            Some(ts) => {
                let parsed = chrono::DateTime::parse_from_rfc3339(&ts).ok();
                match parsed {
                    None => true,
                    Some(dt) => {
                        let elapsed = Utc::now() - dt.with_timezone(&Utc);
                        elapsed.num_seconds() >= interval.max(60)
                    }
                }
            }
        };

        if should_sync {
            let _ = sync_for_account(db_path, &account_id);
        }
    }

    Ok(())
}

fn sync_for_account(db_path: &Path, account_id: &str) -> Result<GithubSyncResultDTO> {
    let _guard = acquire_sync_lock(account_id)
        .ok_or_else(|| anyhow!("sync en progreso para esta cuenta"))?;

    repo::init_db(db_path)?;
    let conn = open_conn(db_path)?;

    let settings = get_settings_row(&conn, account_id)?;
    if !settings.enabled {
        return Ok(GithubSyncResultDTO {
            fetched: 0,
            created_tasks: 0,
            updated_tasks: 0,
            closed_tasks: 0,
            errors: vec![],
        });
    }

    let token = get_token(account_id)?;
    let client = GithubClient::new(token)?;

    let project_id =
        ensure_default_project(&conn, account_id, settings.default_project_id.clone())?;
    let subs = github_list_repo_subscriptions(db_path, account_id)?
        .into_iter()
        .filter(|s| s.enabled)
        .collect::<Vec<_>>();

    let mut result = GithubSyncResultDTO {
        fetched: 0,
        created_tasks: 0,
        updated_tasks: 0,
        closed_tasks: 0,
        errors: vec![],
    };

    for sub in subs {
        if settings.import_pr_reviews {
            match client.search_review_requested_prs(
                &sub.owner,
                &sub.repo,
                get_repo_etag(&conn, &sub.id, "pr")?.as_deref(),
            ) {
                Ok(fetch) => {
                    if !fetch.not_modified {
                        if let Some(etag) = fetch.etag {
                            set_repo_etag(&conn, &sub.id, "pr", Some(etag))?;
                        }
                        for item in fetch.data {
                            let payload_json = Some(
                                serde_json::to_string(&item).unwrap_or_else(|_| "{}".to_string()),
                            );
                            let updated_at_ext = item.updated_at.clone();
                            let candidate = ExternalCandidate {
                                external_key: format!(
                                    "github:{}/{}#{}:pr",
                                    sub.owner, sub.repo, item.number
                                ),
                                kind: "pr".to_string(),
                                url: item.html_url,
                                title: item.title,
                                state: item.state,
                                repo_full: format!("{}/{}", sub.owner, sub.repo),
                                number: Some(item.number),
                                author: item.user.map(|u| u.login),
                                assignee: item.assignee.map(|u| u.login),
                                updated_at_ext,
                                payload_json,
                                is_review_requested: true,
                            };
                            apply_external_candidate(&conn, &project_id, candidate, &mut result)?;
                            result.fetched += 1;
                        }
                    }
                }
                Err(e) => result
                    .errors
                    .push(format!("{}/{} pr: {e}", sub.owner, sub.repo)),
            }
        }

        if settings.import_assigned_issues {
            match client.search_assigned_issues(
                &sub.owner,
                &sub.repo,
                get_repo_etag(&conn, &sub.id, "issues")?.as_deref(),
            ) {
                Ok(fetch) => {
                    if !fetch.not_modified {
                        if let Some(etag) = fetch.etag {
                            set_repo_etag(&conn, &sub.id, "issues", Some(etag))?;
                        }
                        for item in fetch.data {
                            if item.pull_request.is_some() {
                                continue;
                            }
                            let payload_json = Some(
                                serde_json::to_string(&item).unwrap_or_else(|_| "{}".to_string()),
                            );
                            let updated_at_ext = item.updated_at.clone();
                            let candidate = ExternalCandidate {
                                external_key: format!(
                                    "github:{}/{}#{}:issue",
                                    sub.owner, sub.repo, item.number
                                ),
                                kind: "issue".to_string(),
                                url: item.html_url,
                                title: item.title,
                                state: item.state,
                                repo_full: format!("{}/{}", sub.owner, sub.repo),
                                number: Some(item.number),
                                author: item.user.map(|u| u.login),
                                assignee: item.assignee.map(|u| u.login),
                                updated_at_ext,
                                payload_json,
                                is_review_requested: false,
                            };
                            apply_external_candidate(&conn, &project_id, candidate, &mut result)?;
                            result.fetched += 1;
                        }
                    }
                }
                Err(e) => result
                    .errors
                    .push(format!("{}/{} issue: {e}", sub.owner, sub.repo)),
            }
        }

        if settings.import_notifications {
            match client.list_repo_notifications(
                &sub.owner,
                &sub.repo,
                get_repo_etag(&conn, &sub.id, "notifications")?.as_deref(),
            ) {
                Ok(fetch) => {
                    if !fetch.not_modified {
                        if let Some(etag) = fetch.etag {
                            set_repo_etag(&conn, &sub.id, "notifications", Some(etag))?;
                        }
                        for item in fetch.data {
                            let candidate = map_notification_candidate(item);
                            apply_external_candidate(&conn, &project_id, candidate, &mut result)?;
                            result.fetched += 1;
                        }
                    }
                }
                Err(e) => {
                    let msg = e.to_string();
                    if msg.contains("403") || msg.contains("401") {
                        result.errors.push(format!(
                            "notifications requieren PAT classic ({}/{})",
                            sub.owner, sub.repo
                        ));
                    } else {
                        result
                            .errors
                            .push(format!("{}/{} notifications: {e}", sub.owner, sub.repo));
                    }
                }
            }
        }

        conn.execute(
            "UPDATE github_repo_subscriptions SET last_synced_at = ?2 WHERE id = ?1",
            params![sub.id, now_iso()],
        )?;
    }

    Ok(result)
}

fn map_notification_candidate(item: NotificationItem) -> ExternalCandidate {
    let repo_full = item.repository.full_name.clone();
    let payload_json = Some(serde_json::to_string(&item).unwrap_or_else(|_| "{}".to_string()));
    let updated_at_ext = item.updated_at.clone();
    ExternalCandidate {
        external_key: format!("github:{}:notification:{}", repo_full, item.id),
        kind: "notification".to_string(),
        url: item.repository.html_url,
        title: format!("{} [{}]", item.subject.title, item.subject.kind),
        state: "open".to_string(),
        repo_full,
        number: None,
        author: None,
        assignee: None,
        updated_at_ext,
        payload_json,
        is_review_requested: false,
    }
}

fn apply_external_candidate(
    conn: &Connection,
    project_id: &str,
    item: ExternalCandidate,
    result: &mut GithubSyncResultDTO,
) -> Result<()> {
    let now = now_iso();
    let item_id = upsert_external_item(conn, &item, &now)?;

    let linked_task: Option<(String, i64)> = conn
        .query_row(
            "SELECT task_id, user_modified_title FROM task_external_links WHERE external_item_id = ?1",
            params![item_id],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?)),
        )
        .optional()?;

    let desired_status = if item.state == "closed" || item.state == "merged" || item.state == "done"
    {
        "done"
    } else {
        "todo"
    };

    if let Some((task_id, user_modified_title)) = linked_task {
        let mut title = None;
        if user_modified_title == 0 {
            title = Some(rules::build_task_title(&item));
        }

        let mut sql =
            String::from("UPDATE tasks SET status = ?2, updated_at = ?3, external_url = ?4");
        if title.is_some() {
            sql.push_str(", title = ?5");
        }
        sql.push_str(" WHERE id = ?1");

        if let Some(task_title) = title {
            conn.execute(
                &sql,
                params![task_id, desired_status, now, item.url, task_title],
            )?;
        } else {
            conn.execute(&sql, params![task_id, desired_status, now, item.url])?;
        }

        ensure_system_labels(conn, &task_id, &item)?;

        if desired_status == "done" {
            result.closed_tasks += 1;
        } else {
            result.updated_tasks += 1;
        }
        return Ok(());
    }

    let task_id = uuid_v7();
    let title = rules::build_task_title(&item);
    let priority = rules::infer_priority(&item);

    conn.execute(
        "
        INSERT INTO tasks(
            id, project_id, title, status, priority, due_date,
            created_at, updated_at, completed_at, deleted_at, sort_index, recurrence, external_url
        ) VALUES (?1, ?2, ?3, ?4, ?5, NULL, ?6, ?6, NULL, NULL, NULL, NULL, ?7)
        ",
        params![
            task_id,
            project_id,
            title,
            desired_status,
            priority,
            now,
            item.url
        ],
    )?;

    conn.execute(
        "INSERT INTO task_external_links(id, task_id, external_item_id, user_modified_title, created_at) VALUES(?1, ?2, ?3, 0, ?4)",
        params![uuid_v7(), task_id, item_id, now],
    )?;

    ensure_system_labels(conn, &task_id, &item)?;

    result.created_tasks += 1;
    if desired_status == "done" {
        result.closed_tasks += 1;
    }

    Ok(())
}

fn ensure_system_labels(conn: &Connection, task_id: &str, item: &ExternalCandidate) -> Result<()> {
    for label in rules::system_labels(item) {
        let label_id = find_or_create_label(conn, &label)?;
        conn.execute(
            "INSERT OR IGNORE INTO task_labels(task_id, label_id) VALUES(?1, ?2)",
            params![task_id, label_id],
        )?;
    }
    Ok(())
}

fn upsert_external_item(conn: &Connection, item: &ExternalCandidate, now: &str) -> Result<String> {
    let existing: Option<String> = conn
        .query_row(
            "SELECT id FROM external_items WHERE source = 'github' AND external_key = ?1",
            params![item.external_key],
            |row| row.get(0),
        )
        .optional()?;

    if let Some(id) = existing {
        conn.execute(
            "
            UPDATE external_items
            SET kind = ?2,
                url = ?3,
                title = ?4,
                state = ?5,
                repo_full = ?6,
                number = ?7,
                author = ?8,
                assignee = ?9,
                updated_at_ext = ?10,
                payload_json = ?11,
                updated_at = ?12
            WHERE id = ?1
            ",
            params![
                id,
                item.kind,
                item.url,
                item.title,
                item.state,
                item.repo_full,
                item.number,
                item.author,
                item.assignee,
                item.updated_at_ext,
                item.payload_json,
                now,
            ],
        )?;
        return Ok(id);
    }

    let id = uuid_v7();
    conn.execute(
        "
        INSERT INTO external_items(
            id, source, external_key, kind, url, title, state, repo_full,
            number, author, assignee, updated_at_ext, payload_json, created_at, updated_at
        ) VALUES (?1, 'github', ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?13)
        ",
        params![
            id,
            item.external_key,
            item.kind,
            item.url,
            item.title,
            item.state,
            item.repo_full,
            item.number,
            item.author,
            item.assignee,
            item.updated_at_ext,
            item.payload_json,
            now,
        ],
    )?;

    Ok(id)
}

fn ensure_default_project(
    conn: &Connection,
    account_id: &str,
    configured_project: Option<String>,
) -> Result<String> {
    if let Some(project_id) = configured_project {
        let exists: i64 = conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM projects WHERE id = ?1)",
            params![project_id],
            |row| row.get(0),
        )?;
        if exists == 1 {
            return Ok(project_id);
        }
    }

    if let Some(project_id) = conn
        .query_row(
            "SELECT id FROM projects WHERE name = 'GitHub Inbox' COLLATE NOCASE",
            [],
            |row| row.get::<_, String>(0),
        )
        .optional()?
    {
        conn.execute(
            "UPDATE github_settings SET default_project_id = ?2 WHERE account_id = ?1",
            params![account_id, project_id],
        )?;
        return Ok(project_id);
    }

    let id = uuid_v7();
    let now = now_iso();
    conn.execute(
        "INSERT INTO projects(id, name, created_at, updated_at, sort_mode) VALUES(?1, 'GitHub Inbox', ?2, ?2, 'auto')",
        params![id, now],
    )?;
    conn.execute(
        "UPDATE github_settings SET default_project_id = ?2 WHERE account_id = ?1",
        params![account_id, id],
    )?;
    Ok(id)
}

fn find_or_create_label(conn: &Connection, name: &str) -> Result<String> {
    if let Some(label_id) = conn
        .query_row(
            "SELECT id FROM labels WHERE name = ?1 COLLATE NOCASE",
            params![name],
            |row| row.get::<_, String>(0),
        )
        .optional()?
    {
        return Ok(label_id);
    }

    let id = uuid_v7();
    conn.execute(
        "INSERT INTO labels(id, name) VALUES(?1, ?2)",
        params![id, name],
    )?;
    Ok(id)
}

fn get_settings_row(conn: &Connection, account_id: &str) -> Result<GithubSettingsRow> {
    conn.query_row(
        "
        SELECT enabled, sync_interval_sec, import_pr_reviews, import_assigned_issues, import_notifications, default_project_id
        FROM github_settings
        WHERE account_id = ?1
        ",
        params![account_id],
        |row| {
            Ok(GithubSettingsRow {
                enabled: row.get::<_, i64>(0)? == 1,
                sync_interval_sec: row.get(1)?,
                import_pr_reviews: row.get::<_, i64>(2)? == 1,
                import_assigned_issues: row.get::<_, i64>(3)? == 1,
                import_notifications: row.get::<_, i64>(4)? == 1,
                default_project_id: row.get(5)?,
            })
        },
    )
    .optional()?
    .ok_or_else(|| anyhow!("No se encontraron settings GitHub para account_id={account_id}"))
}

fn get_settings_dto(conn: &Connection, account_id: &str) -> Result<GithubSettingsDTO> {
    let row = get_settings_row(conn, account_id)?;
    Ok(GithubSettingsDTO {
        account_id: account_id.to_string(),
        enabled: row.enabled,
        sync_interval_sec: row.sync_interval_sec,
        import_pr_reviews: row.import_pr_reviews,
        import_assigned_issues: row.import_assigned_issues,
        import_notifications: row.import_notifications,
        default_project_id: row.default_project_id,
    })
}

fn get_repo_sub_by_id(conn: &Connection, id: &str) -> Result<RepoSubDTO> {
    conn.query_row(
        "SELECT id, account_id, owner, repo, enabled, last_synced_at FROM github_repo_subscriptions WHERE id = ?1",
        params![id],
        map_repo_sub,
    )
    .optional()?
    .ok_or_else(|| anyhow!("Suscripción no encontrada"))
}

fn map_repo_sub(row: &rusqlite::Row<'_>) -> rusqlite::Result<RepoSubDTO> {
    Ok(RepoSubDTO {
        id: row.get(0)?,
        account_id: row.get(1)?,
        owner: row.get(2)?,
        repo: row.get(3)?,
        enabled: row.get::<_, i64>(4)? == 1,
        last_synced_at: row.get(5)?,
    })
}

fn get_repo_etag(conn: &Connection, sub_id: &str, source: &str) -> Result<Option<String>> {
    let column = match source {
        "pr" => "last_etag_pr",
        "issues" => "last_etag_issues",
        "notifications" => "last_etag_notifications",
        _ => return Err(anyhow!("fuente etag inválida")),
    };

    let sql = format!("SELECT {column} FROM github_repo_subscriptions WHERE id = ?1");
    let etag = conn.query_row(&sql, params![sub_id], |row| row.get::<_, Option<String>>(0))?;
    Ok(etag)
}

fn set_repo_etag(
    conn: &Connection,
    sub_id: &str,
    source: &str,
    value: Option<String>,
) -> Result<()> {
    let column = match source {
        "pr" => "last_etag_pr",
        "issues" => "last_etag_issues",
        "notifications" => "last_etag_notifications",
        _ => return Err(anyhow!("fuente etag inválida")),
    };

    let sql = format!("UPDATE github_repo_subscriptions SET {column} = ?2 WHERE id = ?1");
    conn.execute(&sql, params![sub_id, value])?;
    Ok(())
}

fn open_conn(db_path: &Path) -> Result<Connection> {
    let conn = Connection::open(db_path)?;
    conn.execute("PRAGMA foreign_keys = ON", [])?;
    Ok(conn)
}

fn keyring_account_key(account_id: &str) -> String {
    format!("github:{account_id}")
}

fn set_token(account_id: &str, token: &str) -> Result<()> {
    let entry = Entry::new(KEYRING_SERVICE, &keyring_account_key(account_id))?;
    entry.set_password(token)?;
    Ok(())
}

fn get_token(account_id: &str) -> Result<String> {
    let entry = Entry::new(KEYRING_SERVICE, &keyring_account_key(account_id))?;
    entry
        .get_password()
        .map_err(|e| anyhow!("No se pudo leer token GitHub del keyring: {e}"))
}

fn clear_token(account_id: &str) -> Result<()> {
    let entry = Entry::new(KEYRING_SERVICE, &keyring_account_key(account_id))?;
    let _ = entry.delete_credential();
    Ok(())
}

fn bool_to_i64(value: bool) -> i64 {
    if value {
        1
    } else {
        0
    }
}

fn now_iso() -> String {
    Utc::now().to_rfc3339()
}

fn uuid_v7() -> String {
    Uuid::now_v7().to_string()
}

fn sync_lock_set() -> &'static Mutex<HashSet<String>> {
    SYNC_IN_PROGRESS.get_or_init(|| Mutex::new(HashSet::new()))
}

struct SyncLockGuard {
    account_id: String,
}

impl Drop for SyncLockGuard {
    fn drop(&mut self) {
        if let Ok(mut set) = sync_lock_set().lock() {
            set.remove(&self.account_id);
        }
    }
}

fn acquire_sync_lock(account_id: &str) -> Option<SyncLockGuard> {
    let mut set = sync_lock_set().lock().ok()?;
    if set.contains(account_id) {
        return None;
    }

    set.insert(account_id.to_string());
    Some(SyncLockGuard {
        account_id: account_id.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::integrations::github::models::ExternalCandidate;

    #[test]
    fn sync_lock_no_reentry() {
        let g1 = acquire_sync_lock("a1");
        assert!(g1.is_some());
        let g2 = acquire_sync_lock("a1");
        assert!(g2.is_none());
        drop(g1);
        let g3 = acquire_sync_lock("a1");
        assert!(g3.is_some());
    }

    #[test]
    fn map_title_pr_format() {
        let item = ExternalCandidate {
            external_key: "x".to_string(),
            kind: "pr".to_string(),
            url: "u".to_string(),
            title: "Fix crash".to_string(),
            state: "open".to_string(),
            repo_full: "o/r".to_string(),
            number: Some(123),
            author: None,
            assignee: None,
            updated_at_ext: None,
            payload_json: None,
            is_review_requested: true,
        };

        let title = crate::integrations::github::rules::build_task_title(&item);
        assert_eq!(title, "PR #123 (o/r): Fix crash");
    }
}
