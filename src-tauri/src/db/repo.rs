use std::{collections::HashSet, fs, path::Path};

use anyhow::{anyhow, Result};
use chrono::{Datelike, Duration, Local, Months, NaiveDate, Utc, Weekday};
use rusqlite::{params, Connection, OptionalExtension, ToSql, Transaction};
use uuid::Uuid;

use super::{
    migrations,
    models::{LabelDTO, ParsedQuickAdd, ProjectDTO, TaskDTO, ToggleResultDTO},
};

const TASK_SELECT_BASE: &str = "
SELECT
  t.id,
  t.title,
  t.status,
  t.priority,
  t.due_date,
  t.project_id,
  p.name AS project_name,
  COALESCE(GROUP_CONCAT(DISTINCT l.name), '') AS labels_csv,
  t.updated_at,
  t.recurrence,
  t.sort_index,
  t.external_url
FROM tasks t
LEFT JOIN projects p ON p.id = t.project_id
LEFT JOIN task_labels tl ON tl.task_id = t.id
LEFT JOIN labels l ON l.id = tl.label_id
";

const TASK_GROUP_BY: &str = "
GROUP BY
  t.id,
  t.title,
  t.status,
  t.priority,
  t.due_date,
  t.project_id,
  p.name,
  t.updated_at,
  t.recurrence,
  t.sort_index,
  t.external_url
";

pub fn init_db(db_path: &Path) -> Result<()> {
    if let Some(parent) = db_path.parent() {
        fs::create_dir_all(parent)?;
    }

    maybe_backup_before_new_schema_migrations(db_path)?;
    let conn = open_conn(db_path)?;
    migrations::run(&conn)?;
    Ok(())
}

fn maybe_backup_before_new_schema_migrations(db_path: &Path) -> Result<()> {
    if !db_path.exists() {
        return Ok(());
    }

    let conn = open_conn(db_path)?;
    let pending = migrations::pending_versions(&conn)?;
    drop(conn);

    if pending.is_empty() {
        return Ok(());
    }

    let backups_dir = db_path
        .parent()
        .ok_or_else(|| anyhow!("No se pudo resolver directorio de backup"))?;
    let timestamp = Utc::now().format("%Y%m%d-%H%M%S");
    let backup_name = format!("kitodo.db.bak-{timestamp}");
    let backup_path = backups_dir.join(backup_name);
    fs::copy(db_path, &backup_path)?;
    eprintln!("Backup created at {}", backup_path.display());

    let mut backup_files = fs::read_dir(backups_dir)?
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry
                .file_name()
                .to_string_lossy()
                .starts_with("kitodo.db.bak-")
        })
        .collect::<Vec<_>>();

    backup_files.sort_by_key(|entry| entry.file_name());
    let keep_last = 10usize;
    if backup_files.len() > keep_last {
        let to_remove = backup_files.len() - keep_last;
        for entry in backup_files.into_iter().take(to_remove) {
            let path = entry.path();
            let _ = fs::remove_file(&path);
        }
        eprintln!("Pruned old backups, kept last {keep_last}");
    }
    Ok(())
}

pub fn quick_add(db_path: &Path, input: &str) -> Result<TaskDTO> {
    init_db(db_path)?;
    let parsed = parse_quick_add(input)?;
    let mut conn = open_conn(db_path)?;
    let tx = conn.transaction()?;

    let project_id = match parsed.project.as_deref() {
        Some(project_name) => Some(find_or_create_project(&tx, project_name)?),
        None => None,
    };

    let task_id = uuid_v7();
    let now = now_iso();

    tx.execute(
        "
        INSERT INTO tasks (
          id, project_id, title, status, priority, due_date,
          created_at, updated_at, completed_at, deleted_at, sort_index, recurrence
        ) VALUES (?1, ?2, ?3, 'todo', ?4, ?5, ?6, ?6, NULL, NULL, NULL, ?7)
        ",
        params![
            task_id,
            project_id,
            parsed.title,
            parsed.priority,
            parsed.due_date,
            now,
            parsed.recurrence,
        ],
    )?;

    for label in parsed.labels {
        let label_id = find_or_create_label(&tx, &label)?;
        tx.execute(
            "INSERT OR IGNORE INTO task_labels(task_id, label_id) VALUES(?1, ?2)",
            params![task_id, label_id],
        )?;
    }

    tx.commit()?;
    get_task_by_id(&conn, &task_id)
}

pub fn list_inbox(db_path: &Path, show_done: bool) -> Result<Vec<TaskDTO>> {
    init_db(db_path)?;
    let conn = open_conn(db_path)?;

    let mut sql = String::from(TASK_SELECT_BASE);
    sql.push_str("WHERE t.deleted_at IS NULL AND t.due_date IS NULL");
    if !show_done {
        sql.push_str(" AND t.status = 'todo'");
    }
    sql.push_str(TASK_GROUP_BY);
    sql.push_str("ORDER BY t.priority ASC, t.updated_at DESC");

    query_tasks(&conn, &sql, &[])
}

pub fn list_today(db_path: &Path, show_done: bool) -> Result<Vec<TaskDTO>> {
    init_db(db_path)?;
    let conn = open_conn(db_path)?;
    let today = local_today();

    let mut sql = String::from(TASK_SELECT_BASE);
    sql.push_str("WHERE t.deleted_at IS NULL AND t.due_date = ?1");
    if !show_done {
        sql.push_str(" AND t.status = 'todo'");
    }
    sql.push_str(TASK_GROUP_BY);
    sql.push_str("ORDER BY t.priority ASC, t.updated_at DESC");

    query_tasks(&conn, &sql, &[&today])
}

pub fn list_overdue(db_path: &Path, show_done: bool) -> Result<Vec<TaskDTO>> {
    init_db(db_path)?;
    let conn = open_conn(db_path)?;
    let today = local_today();

    let mut sql = String::from(TASK_SELECT_BASE);
    sql.push_str("WHERE t.deleted_at IS NULL AND t.due_date IS NOT NULL AND t.due_date < ?1");
    if !show_done {
        sql.push_str(" AND t.status = 'todo'");
    }
    sql.push_str(TASK_GROUP_BY);
    sql.push_str("ORDER BY t.due_date ASC, t.priority ASC, t.updated_at DESC");

    query_tasks(&conn, &sql, &[&today])
}

pub fn list_upcoming(db_path: &Path, days: i64, show_done: bool) -> Result<Vec<TaskDTO>> {
    init_db(db_path)?;
    let conn = open_conn(db_path)?;

    let capped_days = days.clamp(0, 365);
    let start = Local::now().date_naive();
    let end = start + Duration::days(capped_days);
    let start_str = start.format("%F").to_string();
    let end_str = end.format("%F").to_string();

    let mut sql = String::from(TASK_SELECT_BASE);
    sql.push_str("WHERE t.deleted_at IS NULL AND t.due_date IS NOT NULL AND t.due_date >= ?1 AND t.due_date <= ?2");
    if !show_done {
        sql.push_str(" AND t.status = 'todo'");
    }
    sql.push_str(TASK_GROUP_BY);
    sql.push_str("ORDER BY t.due_date ASC, t.priority ASC, t.updated_at DESC");

    query_tasks(&conn, &sql, &[&start_str, &end_str])
}

pub fn list_project_tasks(
    db_path: &Path,
    project_id: &str,
    show_done: bool,
) -> Result<Vec<TaskDTO>> {
    init_db(db_path)?;
    let conn = open_conn(db_path)?;
    let mode = get_project_sort_mode_from_conn(&conn, project_id)?;

    let mut sql = String::from(TASK_SELECT_BASE);
    sql.push_str("WHERE t.deleted_at IS NULL AND t.project_id = ?1");
    if !show_done {
        sql.push_str(" AND t.status = 'todo'");
    }
    sql.push_str(TASK_GROUP_BY);

    if mode == "manual" {
        sql.push_str("ORDER BY (t.sort_index IS NULL) ASC, t.sort_index ASC, t.updated_at DESC");
    } else {
        sql.push_str(
            "ORDER BY (t.due_date IS NULL) ASC, t.due_date ASC, t.priority ASC, t.updated_at DESC",
        );
    }

    query_tasks(&conn, &sql, &[&project_id])
}

pub fn list_projects(db_path: &Path) -> Result<Vec<ProjectDTO>> {
    init_db(db_path)?;
    let conn = open_conn(db_path)?;

    let mut stmt =
        conn.prepare("SELECT id, name, sort_mode FROM projects ORDER BY name COLLATE NOCASE ASC")?;
    let rows = stmt.query_map([], |row| {
        Ok(ProjectDTO {
            id: row.get(0)?,
            name: row.get(1)?,
            sort_mode: row
                .get::<_, Option<String>>(2)?
                .unwrap_or_else(|| "auto".to_string()),
        })
    })?;

    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

pub fn list_labels(db_path: &Path) -> Result<Vec<LabelDTO>> {
    init_db(db_path)?;
    let conn = open_conn(db_path)?;

    let mut stmt = conn.prepare("SELECT id, name FROM labels ORDER BY name COLLATE NOCASE ASC")?;
    let rows = stmt.query_map([], |row| {
        Ok(LabelDTO {
            id: row.get(0)?,
            name: row.get(1)?,
        })
    })?;

    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

pub fn get_project_sort_mode(db_path: &Path, project_id: &str) -> Result<String> {
    init_db(db_path)?;
    let conn = open_conn(db_path)?;
    get_project_sort_mode_from_conn(&conn, project_id)
}

pub fn set_project_sort_mode(db_path: &Path, project_id: &str, mode: &str) -> Result<ProjectDTO> {
    if mode != "auto" && mode != "manual" {
        return Err(anyhow!("sort mode inválido: usa auto|manual"));
    }

    init_db(db_path)?;
    let conn = open_conn(db_path)?;

    let rows = conn.execute(
        "UPDATE projects SET sort_mode = ?2, updated_at = ?3 WHERE id = ?1",
        params![project_id, mode, now_iso()],
    )?;

    if rows == 0 {
        return Err(anyhow!("Proyecto no encontrado"));
    }

    get_project_by_id(&conn, project_id)
}

pub fn reorder_project_tasks(
    db_path: &Path,
    project_id: &str,
    ordered_task_ids: Vec<String>,
) -> Result<bool> {
    init_db(db_path)?;
    let mut conn = open_conn(db_path)?;
    let tx = conn.transaction()?;

    let exists = tx.query_row(
        "SELECT EXISTS(SELECT 1 FROM projects WHERE id = ?1)",
        params![project_id],
        |row| row.get::<_, i64>(0),
    )? == 1;

    if !exists {
        return Err(anyhow!("Proyecto no encontrado"));
    }

    tx.execute(
        "UPDATE tasks SET sort_index = NULL WHERE project_id = ?1 AND deleted_at IS NULL",
        params![project_id],
    )?;

    let now = now_iso();
    for (idx, id) in ordered_task_ids.iter().enumerate() {
        tx.execute(
            "
            UPDATE tasks
            SET sort_index = ?3, updated_at = ?4
            WHERE id = ?1 AND project_id = ?2 AND deleted_at IS NULL
            ",
            params![id, project_id, idx as i64, now],
        )?;
    }

    tx.commit()?;
    Ok(true)
}

pub fn reset_project_manual_order(db_path: &Path, project_id: &str) -> Result<bool> {
    init_db(db_path)?;
    let conn = open_conn(db_path)?;
    conn.execute(
        "UPDATE tasks SET sort_index = NULL WHERE project_id = ?1 AND deleted_at IS NULL",
        params![project_id],
    )?;
    Ok(true)
}

pub fn toggle_task(db_path: &Path, id: &str) -> Result<TaskDTO> {
    Ok(toggle_task_with_recurrence(db_path, id)?.updated_task)
}

pub fn toggle_task_with_recurrence(db_path: &Path, id: &str) -> Result<ToggleResultDTO> {
    init_db(db_path)?;
    let mut conn = open_conn(db_path)?;
    let tx = conn.transaction()?;

    let task_row = tx
        .query_row(
            "
            SELECT id, status, recurrence, due_date, title, project_id, priority
            FROM tasks
            WHERE id = ?1 AND deleted_at IS NULL
            ",
            params![id],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, Option<String>>(2)?,
                    row.get::<_, Option<String>>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, Option<String>>(5)?,
                    row.get::<_, i64>(6)?,
                ))
            },
        )
        .optional()?
        .ok_or_else(|| anyhow!("No existe la tarea con id {id}"))?;

    let (task_id, current_status, recurrence, due_date, title, project_id, priority) = task_row;
    let now = now_iso();

    let mut spawned_task_id: Option<String> = None;

    if current_status == "todo" {
        tx.execute(
            "
            UPDATE tasks
            SET status = 'done', completed_at = ?2, updated_at = ?2
            WHERE id = ?1 AND deleted_at IS NULL
            ",
            params![task_id, now],
        )?;

        if let Some(ref recurrence_rule) = recurrence {
            let next_due = calculate_next_due_date(recurrence_rule, due_date.as_deref())?;
            let new_task_id = uuid_v7();
            tx.execute(
                "
                INSERT INTO tasks (
                  id, project_id, title, status, priority, due_date,
                  created_at, updated_at, completed_at, deleted_at, sort_index, recurrence
                ) VALUES (?1, ?2, ?3, 'todo', ?4, ?5, ?6, ?6, NULL, NULL, NULL, ?7)
                ",
                params![
                    new_task_id,
                    project_id,
                    title,
                    priority,
                    next_due,
                    now,
                    recurrence_rule
                ],
            )?;

            // Copy labels to the spawned recurring task.
            tx.execute(
                "
                INSERT OR IGNORE INTO task_labels(task_id, label_id)
                SELECT ?2, label_id FROM task_labels WHERE task_id = ?1
                ",
                params![task_id, new_task_id],
            )?;

            spawned_task_id = Some(new_task_id);
        }
    } else {
        tx.execute(
            "
            UPDATE tasks
            SET status = 'todo', completed_at = NULL, updated_at = ?2
            WHERE id = ?1 AND deleted_at IS NULL
            ",
            params![task_id, now],
        )?;
    }

    tx.commit()?;

    let updated_task = get_task_by_id(&conn, id)?;
    let spawned_task = match spawned_task_id {
        Some(spawned_id) => Some(get_task_by_id(&conn, &spawned_id)?),
        None => None,
    };

    Ok(ToggleResultDTO {
        updated_task,
        spawned_task,
    })
}

pub fn update_task_title(db_path: &Path, id: &str, title: &str) -> Result<TaskDTO> {
    let cleaned_title = title.trim();
    if cleaned_title.is_empty() {
        return Err(anyhow!("El título no puede quedar vacío"));
    }

    init_db(db_path)?;
    let conn = open_conn(db_path)?;

    let rows_affected = conn.execute(
        "
        UPDATE tasks
        SET title = ?2, updated_at = ?3
        WHERE id = ?1 AND deleted_at IS NULL
        ",
        params![id, cleaned_title, now_iso()],
    )?;

    if rows_affected == 0 {
        return Err(anyhow!("No existe la tarea con id {id}"));
    }

    // If this task is linked to an external item, preserve user-custom title on future syncs.
    conn.execute(
        "UPDATE task_external_links SET user_modified_title = 1 WHERE task_id = ?1",
        params![id],
    )?;

    get_task_by_id(&conn, id)
}

pub fn update_task_priority(db_path: &Path, id: &str, priority: i64) -> Result<TaskDTO> {
    if !(1..=4).contains(&priority) {
        return Err(anyhow!("priority debe estar entre 1 y 4"));
    }

    init_db(db_path)?;
    let conn = open_conn(db_path)?;

    let rows_affected = conn.execute(
        "
        UPDATE tasks
        SET priority = ?2, updated_at = ?3
        WHERE id = ?1 AND deleted_at IS NULL
        ",
        params![id, priority, now_iso()],
    )?;

    if rows_affected == 0 {
        return Err(anyhow!("No existe la tarea con id {id}"));
    }

    get_task_by_id(&conn, id)
}

pub fn update_task_due_date(db_path: &Path, id: &str, due_date: Option<String>) -> Result<TaskDTO> {
    let normalized_due = match due_date {
        Some(raw) => {
            let trimmed = raw.trim();
            if trimmed.is_empty() {
                None
            } else {
                NaiveDate::parse_from_str(trimmed, "%Y-%m-%d")
                    .map_err(|_| anyhow!("due_date inválido, usa YYYY-MM-DD"))?;
                Some(trimmed.to_string())
            }
        }
        None => None,
    };

    init_db(db_path)?;
    let conn = open_conn(db_path)?;

    let rows_affected = conn.execute(
        "
        UPDATE tasks
        SET due_date = ?2, updated_at = ?3
        WHERE id = ?1 AND deleted_at IS NULL
        ",
        params![id, normalized_due, now_iso()],
    )?;

    if rows_affected == 0 {
        return Err(anyhow!("No existe la tarea con id {id}"));
    }

    get_task_by_id(&conn, id)
}

pub fn update_task_recurrence(
    db_path: &Path,
    id: &str,
    recurrence: Option<String>,
) -> Result<TaskDTO> {
    let normalized_recurrence = normalize_recurrence(recurrence.as_deref())?;

    init_db(db_path)?;
    let conn = open_conn(db_path)?;

    let rows_affected = conn.execute(
        "
        UPDATE tasks
        SET recurrence = ?2, updated_at = ?3
        WHERE id = ?1 AND deleted_at IS NULL
        ",
        params![id, normalized_recurrence, now_iso()],
    )?;

    if rows_affected == 0 {
        return Err(anyhow!("No existe la tarea con id {id}"));
    }

    get_task_by_id(&conn, id)
}

pub fn move_task_to_project(
    db_path: &Path,
    id: &str,
    project_id: Option<String>,
    project_name_to_create: Option<String>,
) -> Result<TaskDTO> {
    init_db(db_path)?;
    let mut conn = open_conn(db_path)?;
    let tx = conn.transaction()?;

    let mut target_project_id = project_id;

    if let Some(project_name) = project_name_to_create {
        let cleaned = project_name.trim();
        if !cleaned.is_empty() {
            target_project_id = Some(find_or_create_project(&tx, cleaned)?);
        }
    }

    if let Some(project_id_value) = target_project_id.as_deref() {
        let exists = tx.query_row(
            "SELECT EXISTS(SELECT 1 FROM projects WHERE id = ?1)",
            params![project_id_value],
            |row| row.get::<_, i64>(0),
        )? == 1;

        if !exists {
            return Err(anyhow!("El proyecto indicado no existe"));
        }
    }

    let rows_affected = tx.execute(
        "
        UPDATE tasks
        SET project_id = ?2, updated_at = ?3, sort_index = NULL
        WHERE id = ?1 AND deleted_at IS NULL
        ",
        params![id, target_project_id, now_iso()],
    )?;

    if rows_affected == 0 {
        return Err(anyhow!("No existe la tarea con id {id}"));
    }

    tx.commit()?;
    get_task_by_id(&conn, id)
}

pub fn soft_delete_task(db_path: &Path, id: &str) -> Result<bool> {
    init_db(db_path)?;
    let conn = open_conn(db_path)?;
    let rows_affected = conn.execute(
        "
        UPDATE tasks
        SET deleted_at = ?2, updated_at = ?2
        WHERE id = ?1 AND deleted_at IS NULL
        ",
        params![id, now_iso()],
    )?;

    Ok(rows_affected > 0)
}

pub fn restore_task(db_path: &Path, id: &str) -> Result<TaskDTO> {
    init_db(db_path)?;
    let conn = open_conn(db_path)?;

    let rows_affected = conn.execute(
        "
        UPDATE tasks
        SET deleted_at = NULL, updated_at = ?2
        WHERE id = ?1
        ",
        params![id, now_iso()],
    )?;

    if rows_affected == 0 {
        return Err(anyhow!("No existe la tarea con id {id}"));
    }

    get_task_by_id_include_deleted(&conn, id)
}

fn open_conn(db_path: &Path) -> Result<Connection> {
    let conn = Connection::open(db_path)?;
    conn.execute("PRAGMA foreign_keys = ON", [])?;
    Ok(conn)
}

fn parse_labels(labels_csv: &str) -> Vec<String> {
    if labels_csv.trim().is_empty() {
        return Vec::new();
    }

    labels_csv
        .split(',')
        .map(|raw| raw.trim().to_string())
        .filter(|label| !label.is_empty())
        .collect()
}

fn map_task_dto(row: &rusqlite::Row<'_>) -> rusqlite::Result<TaskDTO> {
    let labels_csv: String = row.get(7)?;

    Ok(TaskDTO {
        id: row.get(0)?,
        title: row.get(1)?,
        status: row.get(2)?,
        priority: row.get(3)?,
        due_date: row.get(4)?,
        project_id: row.get(5)?,
        project_name: row.get(6)?,
        labels: parse_labels(&labels_csv),
        updated_at: row.get(8)?,
        recurrence: row.get(9)?,
        sort_index: row.get(10)?,
        external_url: row.get(11)?,
    })
}

fn get_task_by_id(conn: &Connection, id: &str) -> Result<TaskDTO> {
    let mut sql = String::from(TASK_SELECT_BASE);
    sql.push_str("WHERE t.id = ?1 AND t.deleted_at IS NULL");
    sql.push_str(TASK_GROUP_BY);

    let task = conn
        .query_row(&sql, params![id], map_task_dto)
        .optional()?
        .ok_or_else(|| anyhow!("No se pudo recuperar la tarea creada/actualizada"))?;

    Ok(task)
}

fn get_task_by_id_include_deleted(conn: &Connection, id: &str) -> Result<TaskDTO> {
    let mut sql = String::from(TASK_SELECT_BASE);
    sql.push_str("WHERE t.id = ?1");
    sql.push_str(TASK_GROUP_BY);

    let task = conn
        .query_row(&sql, params![id], map_task_dto)
        .optional()?
        .ok_or_else(|| anyhow!("No se pudo recuperar la tarea"))?;

    Ok(task)
}

fn query_tasks(conn: &Connection, sql: &str, bind: &[&dyn ToSql]) -> Result<Vec<TaskDTO>> {
    let mut stmt = conn.prepare(sql)?;
    let rows = stmt.query_map(bind, map_task_dto)?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

fn parse_quick_add(input: &str) -> Result<ParsedQuickAdd> {
    let mut priority = 4;
    let mut project: Option<String> = None;
    let mut labels: Vec<String> = Vec::new();
    let mut title_words: Vec<String> = Vec::new();
    let mut due_date: Option<String> = None;
    let mut recurrence: Option<String> = None;

    let mut tokens = input.split_whitespace().peekable();

    while let Some(token) = tokens.next() {
        if token.starts_with('@') && token.len() > 1 && project.is_none() {
            project = Some(token[1..].trim().to_string());
            continue;
        }

        if token.starts_with('#') && token.len() > 1 {
            labels.push(token[1..].trim().to_string());
            continue;
        }

        if token.len() == 2 && token.starts_with('p') {
            if let Some(level) = token.chars().nth(1).and_then(|c| c.to_digit(10)) {
                if (1..=4).contains(&level) {
                    priority = level as i64;
                    continue;
                }
            }
        }

        if token.eq_ignore_ascii_case("due") {
            let due_raw = tokens
                .next()
                .ok_or_else(|| anyhow!("Falta valor para due (today|tomorrow|YYYY-MM-DD)"))?;
            due_date = parse_due(due_raw)?;
            continue;
        }

        if token.eq_ignore_ascii_case("every") {
            let recurrence_raw = tokens
                .next()
                .ok_or_else(|| anyhow!("Falta valor para recurrence después de 'every'"))?;
            recurrence = parse_recurrence_token(recurrence_raw)?;
            continue;
        }

        title_words.push(token.to_string());
    }

    finish_parse(title_words, project, labels, priority, due_date, recurrence)
}

fn finish_parse(
    title_words: Vec<String>,
    project: Option<String>,
    labels: Vec<String>,
    priority: i64,
    due_date: Option<String>,
    recurrence: Option<String>,
) -> Result<ParsedQuickAdd> {
    let title = title_words.join(" ").trim().to_string();
    if title.is_empty() {
        return Err(anyhow!("No se pudo construir un título válido"));
    }

    let mut seen = HashSet::new();
    let normalized_labels = labels
        .into_iter()
        .map(|s| s.trim().to_lowercase())
        .filter(|s| !s.is_empty())
        .filter(|s| seen.insert(s.clone()))
        .collect::<Vec<_>>();

    let normalized_project = project
        .map(|p| p.trim().to_string())
        .filter(|p| !p.is_empty());

    Ok(ParsedQuickAdd {
        title,
        project: normalized_project,
        labels: normalized_labels,
        priority,
        due_date,
        recurrence,
    })
}

fn parse_due(raw: &str) -> Result<Option<String>> {
    if raw.eq_ignore_ascii_case("today") {
        return Ok(Some(local_today()));
    }

    if raw.eq_ignore_ascii_case("tomorrow") {
        let date = Local::now().date_naive() + Duration::days(1);
        return Ok(Some(date.format("%F").to_string()));
    }

    NaiveDate::parse_from_str(raw, "%Y-%m-%d")
        .map_err(|_| anyhow!("due inválido: usa today, tomorrow o YYYY-MM-DD"))?;

    Ok(Some(raw.to_string()))
}

fn parse_recurrence_token(raw: &str) -> Result<Option<String>> {
    let normalized = raw.trim().to_lowercase();
    if normalized.is_empty() {
        return Ok(None);
    }

    match normalized.as_str() {
        "day" | "daily" => Ok(Some("daily".to_string())),
        "week" | "weekly" => Ok(Some("weekly".to_string())),
        "month" | "monthly" => Ok(Some("monthly".to_string())),
        "mon" | "monday" => Ok(Some("weekday:mon".to_string())),
        "tue" | "tuesday" => Ok(Some("weekday:tue".to_string())),
        "wed" | "wednesday" => Ok(Some("weekday:wed".to_string())),
        "thu" | "thursday" => Ok(Some("weekday:thu".to_string())),
        "fri" | "friday" => Ok(Some("weekday:fri".to_string())),
        "sat" | "saturday" => Ok(Some("weekday:sat".to_string())),
        "sun" | "sunday" => Ok(Some("weekday:sun".to_string())),
        _ => {
            if let Some(interval) = parse_interval_notation(&normalized) {
                return Ok(Some(interval));
            }
            Err(anyhow!(
                "recurrence inválida: usa day|week|month|mon..sun o Nd/Nw/Nm"
            ))
        }
    }
}

fn parse_interval_notation(raw: &str) -> Option<String> {
    if raw.len() < 2 {
        return None;
    }

    let unit = raw.chars().last()?;
    if unit != 'd' && unit != 'w' && unit != 'm' {
        return None;
    }

    let num_str = &raw[..raw.len() - 1];
    let number: i64 = num_str.parse().ok()?;
    if number <= 0 {
        return None;
    }

    Some(format!("interval:{unit}:{number}"))
}

fn normalize_recurrence(raw: Option<&str>) -> Result<Option<String>> {
    let Some(value) = raw else {
        return Ok(None);
    };

    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }

    if ["daily", "weekly", "monthly"].contains(&trimmed) {
        return Ok(Some(trimmed.to_string()));
    }

    if trimmed.starts_with("weekday:") {
        let day = trimmed.trim_start_matches("weekday:");
        if ["mon", "tue", "wed", "thu", "fri", "sat", "sun"].contains(&day) {
            return Ok(Some(trimmed.to_string()));
        }
    }

    if trimmed.starts_with("interval:") {
        let parts = trimmed.split(':').collect::<Vec<_>>();
        if parts.len() == 3 {
            let unit = parts[1];
            let n = parts[2].parse::<i64>().ok();
            if ["d", "w", "m"].contains(&unit) && n.map(|v| v > 0).unwrap_or(false) {
                return Ok(Some(trimmed.to_string()));
            }
        }
    }

    Err(anyhow!("recurrence inválida"))
}

fn calculate_next_due_date(recurrence: &str, current_due_date: Option<&str>) -> Result<String> {
    let today = Local::now().date_naive();
    let base_date = match current_due_date {
        Some(date_str) => NaiveDate::parse_from_str(date_str, "%Y-%m-%d").unwrap_or(today),
        None => today,
    };

    let recurrence =
        normalize_recurrence(Some(recurrence))?.ok_or_else(|| anyhow!("recurrence vacía"))?;

    let next = if recurrence == "daily" {
        base_date + Duration::days(1)
    } else if recurrence == "weekly" {
        base_date + Duration::days(7)
    } else if recurrence == "monthly" {
        add_months_safe(base_date, 1)?
    } else if recurrence.starts_with("weekday:") {
        let w = parse_weekday(recurrence.trim_start_matches("weekday:"))?;
        next_weekday_after(today, w)
    } else if recurrence.starts_with("interval:") {
        let parts = recurrence.split(':').collect::<Vec<_>>();
        let unit = parts[1];
        let n: i64 = parts[2].parse().map_err(|_| anyhow!("interval inválido"))?;
        match unit {
            "d" => base_date + Duration::days(n),
            "w" => base_date + Duration::days(n * 7),
            "m" => add_months_safe(base_date, n as u32)?,
            _ => return Err(anyhow!("unidad de interval inválida")),
        }
    } else {
        return Err(anyhow!("recurrence no soportada"));
    };

    Ok(next.format("%F").to_string())
}

fn add_months_safe(date: NaiveDate, months: u32) -> Result<NaiveDate> {
    if let Some(candidate) = date.checked_add_months(Months::new(months)) {
        return Ok(candidate);
    }

    let mut y = date.year();
    let mut m = date.month() as i32 + months as i32;
    while m > 12 {
        y += 1;
        m -= 12;
    }

    let mut day = date.day();
    loop {
        if let Some(d) = NaiveDate::from_ymd_opt(y, m as u32, day) {
            return Ok(d);
        }
        if day == 1 {
            return Err(anyhow!("No se pudo calcular fecha mensual"));
        }
        day -= 1;
    }
}

fn parse_weekday(raw: &str) -> Result<Weekday> {
    match raw {
        "mon" => Ok(Weekday::Mon),
        "tue" => Ok(Weekday::Tue),
        "wed" => Ok(Weekday::Wed),
        "thu" => Ok(Weekday::Thu),
        "fri" => Ok(Weekday::Fri),
        "sat" => Ok(Weekday::Sat),
        "sun" => Ok(Weekday::Sun),
        _ => Err(anyhow!("weekday inválido")),
    }
}

fn next_weekday_after(start: NaiveDate, target: Weekday) -> NaiveDate {
    let mut cursor = start + Duration::days(1);
    loop {
        if cursor.weekday() == target {
            return cursor;
        }
        cursor += Duration::days(1);
    }
}

fn get_project_sort_mode_from_conn(conn: &Connection, project_id: &str) -> Result<String> {
    let mode = conn
        .query_row(
            "SELECT sort_mode FROM projects WHERE id = ?1",
            params![project_id],
            |row| row.get::<_, Option<String>>(0),
        )
        .optional()?
        .flatten()
        .ok_or_else(|| anyhow!("Proyecto no encontrado"))?;

    if mode == "manual" {
        Ok("manual".to_string())
    } else {
        Ok("auto".to_string())
    }
}

fn get_project_by_id(conn: &Connection, project_id: &str) -> Result<ProjectDTO> {
    conn.query_row(
        "SELECT id, name, sort_mode FROM projects WHERE id = ?1",
        params![project_id],
        |row| {
            Ok(ProjectDTO {
                id: row.get(0)?,
                name: row.get(1)?,
                sort_mode: row
                    .get::<_, Option<String>>(2)?
                    .unwrap_or_else(|| "auto".to_string()),
            })
        },
    )
    .optional()?
    .ok_or_else(|| anyhow!("Proyecto no encontrado"))
}

fn find_or_create_project(tx: &Transaction<'_>, name: &str) -> Result<String> {
    if let Some(existing) = tx
        .query_row(
            "SELECT id FROM projects WHERE name = ?1 COLLATE NOCASE",
            params![name],
            |row| row.get::<_, String>(0),
        )
        .optional()?
    {
        return Ok(existing);
    }

    let id = uuid_v7();
    let now = now_iso();
    tx.execute(
        "
        INSERT INTO projects(id, name, created_at, updated_at, sort_mode)
        VALUES (?1, ?2, ?3, ?3, 'auto')
        ",
        params![id, name, now],
    )?;

    Ok(id)
}

fn find_or_create_label(tx: &Transaction<'_>, name: &str) -> Result<String> {
    if let Some(existing) = tx
        .query_row(
            "SELECT id FROM labels WHERE name = ?1 COLLATE NOCASE",
            params![name],
            |row| row.get::<_, String>(0),
        )
        .optional()?
    {
        return Ok(existing);
    }

    let id = uuid_v7();
    tx.execute(
        "INSERT INTO labels(id, name) VALUES(?1, ?2)",
        params![id, name],
    )?;

    Ok(id)
}

fn now_iso() -> String {
    Utc::now().to_rfc3339()
}

fn local_today() -> String {
    Local::now().date_naive().format("%F").to_string()
}

fn uuid_v7() -> String {
    Uuid::now_v7().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recurrence_parser_supports_basic_tokens() {
        assert_eq!(
            parse_recurrence_token("day").unwrap(),
            Some("daily".to_string())
        );
        assert_eq!(
            parse_recurrence_token("week").unwrap(),
            Some("weekly".to_string())
        );
        assert_eq!(
            parse_recurrence_token("month").unwrap(),
            Some("monthly".to_string())
        );
        assert_eq!(
            parse_recurrence_token("mon").unwrap(),
            Some("weekday:mon".to_string())
        );
        assert_eq!(
            parse_recurrence_token("2d").unwrap(),
            Some("interval:d:2".to_string())
        );
        assert!(parse_recurrence_token("everyday").is_err());
    }

    #[test]
    fn next_due_date_daily_weekly_interval() {
        assert_eq!(
            calculate_next_due_date("daily", Some("2026-03-10")).unwrap(),
            "2026-03-11"
        );
        assert_eq!(
            calculate_next_due_date("weekly", Some("2026-03-10")).unwrap(),
            "2026-03-17"
        );
        assert_eq!(
            calculate_next_due_date("interval:d:3", Some("2026-03-10")).unwrap(),
            "2026-03-13"
        );
    }

    #[test]
    fn monthly_clamp_end_of_month() {
        assert_eq!(
            calculate_next_due_date("monthly", Some("2026-01-31")).unwrap(),
            "2026-02-28"
        );
    }

    #[test]
    fn weekday_next_occurrence() {
        let monday = NaiveDate::from_ymd_opt(2026, 3, 2).unwrap(); // Monday
        let next_monday = next_weekday_after(monday, Weekday::Mon);
        assert_eq!(next_monday.format("%F").to_string(), "2026-03-09");
    }

    #[test]
    fn interval_days_from_due() {
        assert_eq!(
            calculate_next_due_date("interval:d:2", Some("2026-03-10")).unwrap(),
            "2026-03-12"
        );
    }

    #[test]
    fn interval_weeks() {
        assert_eq!(
            calculate_next_due_date("interval:w:3", Some("2026-03-10")).unwrap(),
            "2026-03-31"
        );
    }

    #[test]
    fn interval_months_clamp() {
        assert_eq!(
            calculate_next_due_date("interval:m:1", Some("2026-01-31")).unwrap(),
            "2026-02-28"
        );
    }
}
