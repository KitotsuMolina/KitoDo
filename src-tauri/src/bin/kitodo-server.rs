use std::{
    net::SocketAddr,
    path::{Path, PathBuf},
};

use axum::{
    extract::{Path as AxumPath, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use clap::Parser;
use kitodo::{
    db::repo,
    integrations::github::sync::{self, GithubSettingsPatch},
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Clone)]
struct AppState {
    db_path: PathBuf,
}

#[derive(Parser)]
struct Cli {
    #[arg(long, default_value = "127.0.0.1")]
    host: String,
    #[arg(long)]
    port: u16,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ErrorResponse {
    error: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct MoveTaskPayload {
    id: String,
    project_id: Option<String>,
    project_name_to_create: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GithubSettingsPatchInput {
    enabled: Option<bool>,
    sync_interval_sec: Option<i64>,
    import_pr_reviews: Option<bool>,
    import_assigned_issues: Option<bool>,
    import_notifications: Option<bool>,
    default_project_id: Option<Option<String>>,
}

type AppResult<T> = std::result::Result<T, String>;

#[tokio::main]
async fn main() -> AppResult<()> {
    let cli = Cli::parse();
    let db_path = resolve_db_path()?;
    repo::init_db(&db_path).map_err(|e| e.to_string())?;
    sync::spawn_poller(db_path.clone());

    let app = Router::new()
        .route("/health", get(health))
        .route("/invoke/:command", post(invoke))
        .with_state(AppState { db_path });

    let addr: SocketAddr = format!("{}:{}", cli.host, cli.port)
        .parse()
        .map_err(|e| format!("Dirección inválida: {e}"))?;

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .map_err(|e| format!("No se pudo abrir el puerto {addr}: {e}"))?;

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .map_err(|e| format!("Error ejecutando kitodo-server: {e}"))
}

async fn health() -> impl IntoResponse {
    Json(json!({ "ok": true }))
}

async fn invoke(
    State(state): State<AppState>,
    AxumPath(command): AxumPath<String>,
    Json(payload): Json<Value>,
) -> impl IntoResponse {
    match invoke_inner(&state.db_path, &command, payload) {
        Ok(value) => (StatusCode::OK, Json(value)).into_response(),
        Err(error) => (
            StatusCode::BAD_REQUEST,
            Json(json!(ErrorResponse { error })),
        )
            .into_response(),
    }
}

fn invoke_inner(db_path: &Path, command: &str, payload: Value) -> Result<Value, String> {
    repo::init_db(db_path).map_err(|e| e.to_string())?;

    match command {
        "init_db" => ok(true),
        "quick_add" => {
            let input = required_string(&payload, "input")?;
            ok(repo::quick_add(db_path, &input).map_err(|e| e.to_string())?)
        }
        "list_inbox" => {
            let show_done = optional_bool(&payload, "showDone").unwrap_or(false);
            ok(repo::list_inbox(db_path, show_done).map_err(|e| e.to_string())?)
        }
        "list_today" => {
            let show_done = optional_bool(&payload, "showDone").unwrap_or(false);
            ok(repo::list_today(db_path, show_done).map_err(|e| e.to_string())?)
        }
        "list_overdue" => {
            let show_done = optional_bool(&payload, "showDone").unwrap_or(false);
            ok(repo::list_overdue(db_path, show_done).map_err(|e| e.to_string())?)
        }
        "list_upcoming" => {
            let days = required_i64(&payload, "days")?;
            let show_done = optional_bool(&payload, "showDone").unwrap_or(false);
            ok(repo::list_upcoming(db_path, days, show_done).map_err(|e| e.to_string())?)
        }
        "list_project_tasks" => {
            let project_id = required_string(&payload, "projectId")?;
            let show_done = optional_bool(&payload, "showDone").unwrap_or(false);
            ok(repo::list_project_tasks(db_path, &project_id, show_done).map_err(|e| e.to_string())?)
        }
        "list_projects" => ok(repo::list_projects(db_path).map_err(|e| e.to_string())?),
        "list_labels" => ok(repo::list_labels(db_path).map_err(|e| e.to_string())?),
        "get_project_sort_mode" => {
            let project_id = required_string(&payload, "projectId")?;
            ok(repo::get_project_sort_mode(db_path, &project_id).map_err(|e| e.to_string())?)
        }
        "set_project_sort_mode" => {
            let project_id = required_string(&payload, "projectId")?;
            let mode = required_string(&payload, "mode")?;
            ok(repo::set_project_sort_mode(db_path, &project_id, &mode).map_err(|e| e.to_string())?)
        }
        "reorder_project_tasks" => {
            let project_id = required_string(&payload, "projectId")?;
            let ordered_task_ids = required_string_vec(&payload, "orderedTaskIds")?;
            ok(repo::reorder_project_tasks(db_path, &project_id, ordered_task_ids).map_err(|e| e.to_string())?)
        }
        "reset_project_manual_order" => {
            let project_id = required_string(&payload, "projectId")?;
            ok(repo::reset_project_manual_order(db_path, &project_id).map_err(|e| e.to_string())?)
        }
        "toggle_task" => {
            let id = required_string(&payload, "id")?;
            ok(repo::toggle_task(db_path, &id).map_err(|e| e.to_string())?)
        }
        "toggle_task_with_recurrence" => {
            let id = required_string(&payload, "id")?;
            ok(repo::toggle_task_with_recurrence(db_path, &id).map_err(|e| e.to_string())?)
        }
        "update_task_title" => {
            let id = required_string(&payload, "id")?;
            let title = required_string(&payload, "title")?;
            ok(repo::update_task_title(db_path, &id, &title).map_err(|e| e.to_string())?)
        }
        "update_task_priority" => {
            let id = required_string(&payload, "id")?;
            let priority = required_i64(&payload, "priority")?;
            ok(repo::update_task_priority(db_path, &id, priority).map_err(|e| e.to_string())?)
        }
        "update_task_due_date" => {
            let id = required_string(&payload, "id")?;
            let due_date = optional_nullable_string(&payload, "dueDate");
            ok(repo::update_task_due_date(db_path, &id, due_date).map_err(|e| e.to_string())?)
        }
        "update_task_recurrence" => {
            let id = required_string(&payload, "id")?;
            let recurrence = optional_nullable_string(&payload, "recurrence");
            ok(repo::update_task_recurrence(db_path, &id, recurrence).map_err(|e| e.to_string())?)
        }
        "move_task_to_project" => {
            let move_payload: MoveTaskPayload = serde_json::from_value(payload).map_err(|e| e.to_string())?;
            ok(
                repo::move_task_to_project(
                    db_path,
                    &move_payload.id,
                    move_payload.project_id,
                    move_payload.project_name_to_create,
                )
                .map_err(|e| e.to_string())?,
            )
        }
        "soft_delete_task" => {
            let id = required_string(&payload, "id")?;
            ok(repo::soft_delete_task(db_path, &id).map_err(|e| e.to_string())?)
        }
        "restore_task" => {
            let id = required_string(&payload, "id")?;
            ok(repo::restore_task(db_path, &id).map_err(|e| e.to_string())?)
        }
        "export_backup_json" => ok(repo::export_backup_json(db_path).map_err(|e| e.to_string())?),
        "import_backup_json" => {
            let backup_json = required_string(&payload, "json")?;
            ok(repo::import_backup_json(db_path, &backup_json).map_err(|e| e.to_string())?)
        }
        "github_connect" => {
            let token = required_string(&payload, "token")?;
            ok(sync::github_connect(db_path, &token).map_err(|e| e.to_string())?)
        }
        "github_disconnect" => {
            let account_id = required_string(&payload, "accountId")?;
            ok(sync::github_disconnect(db_path, &account_id).map_err(|e| e.to_string())?)
        }
        "github_list_accounts" => ok(sync::github_list_accounts(db_path).map_err(|e| e.to_string())?),
        "github_get_settings" => {
            let account_id = required_string(&payload, "accountId")?;
            ok(sync::github_get_settings(db_path, &account_id).map_err(|e| e.to_string())?)
        }
        "github_set_settings" => {
            let account_id = required_string(&payload, "accountId")?;
            let settings_patch: GithubSettingsPatchInput =
                serde_json::from_value(required_value(&payload, "settingsPatch")?).map_err(|e| e.to_string())?;
            let patch = GithubSettingsPatch {
                enabled: settings_patch.enabled,
                sync_interval_sec: settings_patch.sync_interval_sec,
                import_pr_reviews: settings_patch.import_pr_reviews,
                import_assigned_issues: settings_patch.import_assigned_issues,
                import_notifications: settings_patch.import_notifications,
                default_project_id: settings_patch.default_project_id,
            };
            ok(sync::github_set_settings(db_path, &account_id, patch).map_err(|e| e.to_string())?)
        }
        "github_list_repos" => {
            let account_id = required_string(&payload, "accountId")?;
            ok(sync::github_list_repos(db_path, &account_id).map_err(|e| e.to_string())?)
        }
        "github_add_repo_subscription" => {
            let account_id = required_string(&payload, "accountId")?;
            let owner = required_string(&payload, "owner")?;
            let repo_name = required_string(&payload, "repo")?;
            ok(sync::github_add_repo_subscription(db_path, &account_id, &owner, &repo_name).map_err(|e| e.to_string())?)
        }
        "github_remove_repo_subscription" => {
            let id = required_string(&payload, "id")?;
            ok(sync::github_remove_repo_subscription(db_path, &id).map_err(|e| e.to_string())?)
        }
        "github_toggle_repo_subscription" => {
            let id = required_string(&payload, "id")?;
            let enabled = required_bool(&payload, "enabled")?;
            ok(sync::github_toggle_repo_subscription(db_path, &id, enabled).map_err(|e| e.to_string())?)
        }
        "github_list_repo_subscriptions" => {
            let account_id = required_string(&payload, "accountId")?;
            ok(sync::github_list_repo_subscriptions(db_path, &account_id).map_err(|e| e.to_string())?)
        }
        "github_sync_now" => {
            let account_id = required_string(&payload, "accountId")?;
            ok(sync::github_sync_now(db_path, &account_id).map_err(|e| e.to_string())?)
        }
        "github_get_status" => {
            let account_id = required_string(&payload, "accountId")?;
            ok(sync::github_get_status(db_path, &account_id).map_err(|e| e.to_string())?)
        }
        "github_list_external_items" => {
            let limit = optional_i64(&payload, "limit");
            ok(sync::github_list_external_items(db_path, limit.unwrap_or(100)).map_err(|e| e.to_string())?)
        }
        _ => Err(format!("Comando desconocido: {command}")),
    }
}

fn resolve_db_path() -> AppResult<PathBuf> {
    let mut dir = dirs::data_dir().ok_or_else(|| "No se pudo resolver data_dir".to_string())?;
    dir.push("kitodo");
    std::fs::create_dir_all(&dir)
        .map_err(|e| format!("No se pudo crear directorio de datos {:?}: {e}", dir))?;
    Ok(dir.join("kitodo.db"))
}

fn ok<T: Serialize>(value: T) -> AppResult<Value> {
    serde_json::to_value(value).map_err(|e| e.to_string())
}

fn required_value(payload: &Value, key: &str) -> AppResult<Value> {
    payload
        .get(key)
        .cloned()
        .ok_or_else(|| format!("Falta campo requerido: {key}"))
}

fn required_string(payload: &Value, key: &str) -> AppResult<String> {
    payload
        .get(key)
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
        .ok_or_else(|| format!("Campo inválido: {key}"))
}

fn required_bool(payload: &Value, key: &str) -> AppResult<bool> {
    payload
        .get(key)
        .and_then(Value::as_bool)
        .ok_or_else(|| format!("Campo inválido: {key}"))
}

fn required_i64(payload: &Value, key: &str) -> AppResult<i64> {
    payload
        .get(key)
        .and_then(Value::as_i64)
        .ok_or_else(|| format!("Campo inválido: {key}"))
}

fn optional_i64(payload: &Value, key: &str) -> Option<i64> {
    payload.get(key).and_then(Value::as_i64)
}

fn optional_bool(payload: &Value, key: &str) -> Option<bool> {
    payload.get(key).and_then(Value::as_bool)
}

fn optional_nullable_string(payload: &Value, key: &str) -> Option<String> {
    match payload.get(key) {
        Some(Value::String(value)) => Some(value.clone()),
        Some(Value::Null) | None => None,
        _ => None,
    }
}

fn required_string_vec(payload: &Value, key: &str) -> AppResult<Vec<String>> {
    let values = payload
        .get(key)
        .and_then(Value::as_array)
        .ok_or_else(|| format!("Campo inválido: {key}"))?;

    values
        .iter()
        .map(|value| {
            value
                .as_str()
                .map(ToOwned::to_owned)
                .ok_or_else(|| format!("Campo inválido: {key}"))
        })
        .collect()
}

async fn shutdown_signal() {
    let _ = tokio::signal::ctrl_c().await;
}
