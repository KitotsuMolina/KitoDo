use tauri::State;

use crate::AppState;
use kitodo::db::{
    models::{ImportResultDTO, LabelDTO, ProjectDTO, TaskDTO, ToggleResultDTO},
    repo,
};

#[tauri::command]
pub fn init_db(state: State<'_, AppState>) -> Result<bool, String> {
    repo::init_db(&state.db_path)
        .map(|_| true)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn quick_add(state: State<'_, AppState>, input: String) -> Result<TaskDTO, String> {
    repo::quick_add(&state.db_path, &input).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_inbox(state: State<'_, AppState>, show_done: bool) -> Result<Vec<TaskDTO>, String> {
    repo::list_inbox(&state.db_path, show_done).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_today(state: State<'_, AppState>, show_done: bool) -> Result<Vec<TaskDTO>, String> {
    repo::list_today(&state.db_path, show_done).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_overdue(state: State<'_, AppState>, show_done: bool) -> Result<Vec<TaskDTO>, String> {
    repo::list_overdue(&state.db_path, show_done).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_upcoming(
    state: State<'_, AppState>,
    days: i64,
    show_done: bool,
) -> Result<Vec<TaskDTO>, String> {
    repo::list_upcoming(&state.db_path, days, show_done).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_project_tasks(
    state: State<'_, AppState>,
    project_id: String,
    show_done: bool,
) -> Result<Vec<TaskDTO>, String> {
    repo::list_project_tasks(&state.db_path, &project_id, show_done).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_projects(state: State<'_, AppState>) -> Result<Vec<ProjectDTO>, String> {
    repo::list_projects(&state.db_path).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_labels(state: State<'_, AppState>) -> Result<Vec<LabelDTO>, String> {
    repo::list_labels(&state.db_path).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_project_sort_mode(
    state: State<'_, AppState>,
    project_id: String,
) -> Result<String, String> {
    repo::get_project_sort_mode(&state.db_path, &project_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_project_sort_mode(
    state: State<'_, AppState>,
    project_id: String,
    mode: String,
) -> Result<ProjectDTO, String> {
    repo::set_project_sort_mode(&state.db_path, &project_id, &mode).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn reorder_project_tasks(
    state: State<'_, AppState>,
    project_id: String,
    ordered_task_ids: Vec<String>,
) -> Result<bool, String> {
    repo::reorder_project_tasks(&state.db_path, &project_id, ordered_task_ids)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn reset_project_manual_order(
    state: State<'_, AppState>,
    project_id: String,
) -> Result<bool, String> {
    repo::reset_project_manual_order(&state.db_path, &project_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn toggle_task(state: State<'_, AppState>, id: String) -> Result<TaskDTO, String> {
    repo::toggle_task(&state.db_path, &id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn toggle_task_with_recurrence(
    state: State<'_, AppState>,
    id: String,
) -> Result<ToggleResultDTO, String> {
    repo::toggle_task_with_recurrence(&state.db_path, &id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_task_title(
    state: State<'_, AppState>,
    id: String,
    title: String,
) -> Result<TaskDTO, String> {
    repo::update_task_title(&state.db_path, &id, &title).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_task_priority(
    state: State<'_, AppState>,
    id: String,
    priority: i64,
) -> Result<TaskDTO, String> {
    repo::update_task_priority(&state.db_path, &id, priority).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_task_due_date(
    state: State<'_, AppState>,
    id: String,
    due_date: Option<String>,
) -> Result<TaskDTO, String> {
    repo::update_task_due_date(&state.db_path, &id, due_date).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_task_recurrence(
    state: State<'_, AppState>,
    id: String,
    recurrence: Option<String>,
) -> Result<TaskDTO, String> {
    repo::update_task_recurrence(&state.db_path, &id, recurrence).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn move_task_to_project(
    state: State<'_, AppState>,
    id: String,
    project_id: Option<String>,
    project_name_to_create: Option<String>,
) -> Result<TaskDTO, String> {
    repo::move_task_to_project(&state.db_path, &id, project_id, project_name_to_create)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn soft_delete_task(state: State<'_, AppState>, id: String) -> Result<bool, String> {
    repo::soft_delete_task(&state.db_path, &id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn restore_task(state: State<'_, AppState>, id: String) -> Result<TaskDTO, String> {
    repo::restore_task(&state.db_path, &id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn export_backup_json(state: State<'_, AppState>) -> Result<String, String> {
    repo::export_backup_json(&state.db_path).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn import_backup_json(
    state: State<'_, AppState>,
    json: String,
) -> Result<ImportResultDTO, String> {
    repo::import_backup_json(&state.db_path, &json).map_err(|e| e.to_string())
}
