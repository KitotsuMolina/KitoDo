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
