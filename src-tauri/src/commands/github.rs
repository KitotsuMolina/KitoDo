use tauri::State;

use crate::AppState;
use kitodo::{
    db::models::{
        GithubAccountDTO, GithubExternalItemDTO, GithubSettingsDTO, GithubStatusDTO,
        GithubSyncResultDTO, RepoDTO, RepoSubDTO,
    },
    integrations::github::sync::{self, GithubSettingsPatch},
};

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GithubSettingsPatchInput {
    pub enabled: Option<bool>,
    pub sync_interval_sec: Option<i64>,
    pub import_pr_reviews: Option<bool>,
    pub import_assigned_issues: Option<bool>,
    pub import_notifications: Option<bool>,
    pub default_project_id: Option<Option<String>>,
}

#[tauri::command]
pub fn github_connect(
    state: State<'_, AppState>,
    token: String,
) -> Result<GithubAccountDTO, String> {
    sync::github_connect(&state.db_path, &token).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn github_disconnect(state: State<'_, AppState>, account_id: String) -> Result<bool, String> {
    sync::github_disconnect(&state.db_path, &account_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn github_list_accounts(state: State<'_, AppState>) -> Result<Vec<GithubAccountDTO>, String> {
    sync::github_list_accounts(&state.db_path).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn github_get_settings(
    state: State<'_, AppState>,
    account_id: String,
) -> Result<GithubSettingsDTO, String> {
    sync::github_get_settings(&state.db_path, &account_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn github_set_settings(
    state: State<'_, AppState>,
    account_id: String,
    settings_patch: GithubSettingsPatchInput,
) -> Result<GithubSettingsDTO, String> {
    let patch = GithubSettingsPatch {
        enabled: settings_patch.enabled,
        sync_interval_sec: settings_patch.sync_interval_sec,
        import_pr_reviews: settings_patch.import_pr_reviews,
        import_assigned_issues: settings_patch.import_assigned_issues,
        import_notifications: settings_patch.import_notifications,
        default_project_id: settings_patch.default_project_id,
    };

    sync::github_set_settings(&state.db_path, &account_id, patch).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn github_list_repos(
    state: State<'_, AppState>,
    account_id: String,
) -> Result<Vec<RepoDTO>, String> {
    sync::github_list_repos(&state.db_path, &account_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn github_add_repo_subscription(
    state: State<'_, AppState>,
    account_id: String,
    owner: String,
    repo: String,
) -> Result<RepoSubDTO, String> {
    sync::github_add_repo_subscription(&state.db_path, &account_id, &owner, &repo)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn github_remove_repo_subscription(
    state: State<'_, AppState>,
    id: String,
) -> Result<bool, String> {
    sync::github_remove_repo_subscription(&state.db_path, &id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn github_toggle_repo_subscription(
    state: State<'_, AppState>,
    id: String,
    enabled: bool,
) -> Result<RepoSubDTO, String> {
    sync::github_toggle_repo_subscription(&state.db_path, &id, enabled).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn github_list_repo_subscriptions(
    state: State<'_, AppState>,
    account_id: String,
) -> Result<Vec<RepoSubDTO>, String> {
    sync::github_list_repo_subscriptions(&state.db_path, &account_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn github_sync_now(
    state: State<'_, AppState>,
    account_id: String,
) -> Result<GithubSyncResultDTO, String> {
    sync::github_sync_now(&state.db_path, &account_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn github_get_status(
    state: State<'_, AppState>,
    account_id: String,
) -> Result<GithubStatusDTO, String> {
    sync::github_get_status(&state.db_path, &account_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn github_list_external_items(
    state: State<'_, AppState>,
    limit: Option<i64>,
) -> Result<Vec<GithubExternalItemDTO>, String> {
    sync::github_list_external_items(&state.db_path, limit.unwrap_or(100))
        .map_err(|e| e.to_string())
}
