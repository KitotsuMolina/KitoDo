use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskDTO {
    pub id: String,
    pub title: String,
    pub status: String,
    pub priority: i64,
    pub due_date: Option<String>,
    pub project_id: Option<String>,
    pub project_name: Option<String>,
    pub labels: Vec<String>,
    pub updated_at: String,
    pub recurrence: Option<String>,
    pub sort_index: Option<i64>,
    pub external_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GithubAccountDTO {
    pub account_id: String,
    pub username: String,
    pub token_kind: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GithubSettingsDTO {
    pub account_id: String,
    pub enabled: bool,
    pub sync_interval_sec: i64,
    pub import_pr_reviews: bool,
    pub import_assigned_issues: bool,
    pub import_notifications: bool,
    pub default_project_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RepoSubDTO {
    pub id: String,
    pub account_id: String,
    pub owner: String,
    pub repo: String,
    pub enabled: bool,
    pub last_synced_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GithubSyncResultDTO {
    pub fetched: i64,
    pub created_tasks: i64,
    pub updated_tasks: i64,
    pub closed_tasks: i64,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GithubStatusDTO {
    pub account_id: String,
    pub username: String,
    pub enabled: bool,
    pub last_sync_at: Option<String>,
    pub last_error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RepoDTO {
    pub owner: String,
    pub repo: String,
    pub full_name: String,
    pub private: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GithubExternalItemDTO {
    pub id: String,
    pub kind: String,
    pub url: String,
    pub title: String,
    pub state: String,
    pub repo_full: String,
    pub number: Option<i64>,
    pub updated_at_ext: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectDTO {
    pub id: String,
    pub name: String,
    pub sort_mode: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LabelDTO {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct ParsedQuickAdd {
    pub title: String,
    pub project: Option<String>,
    pub labels: Vec<String>,
    pub priority: i64,
    pub due_date: Option<String>,
    pub recurrence: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToggleResultDTO {
    pub updated_task: TaskDTO,
    pub spawned_task: Option<TaskDTO>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupExportDTO {
    pub schema_version: i64,
    pub exported_at: String,
    pub projects: Vec<BackupProjectDTO>,
    pub tasks: Vec<BackupTaskDTO>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupProjectDTO {
    pub id: String,
    pub name: String,
    pub sort_mode: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupTaskDTO {
    pub id: String,
    pub title: String,
    pub status: String,
    pub priority: i64,
    pub due_date: Option<String>,
    pub project_id: Option<String>,
    pub labels: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
    pub completed_at: Option<String>,
    pub recurrence: Option<String>,
    pub sort_index: Option<i64>,
    pub external_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportResultDTO {
    pub imported_projects: i64,
    pub created_tasks: i64,
    pub updated_tasks: i64,
    pub linked_labels: i64,
}
