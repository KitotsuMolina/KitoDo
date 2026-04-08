mod commands;

use std::{fs, path::PathBuf};

use tauri::Manager;

pub struct AppState {
    pub db_path: PathBuf,
}

fn resolve_db_path(app: &tauri::App) -> Result<PathBuf, String> {
    let data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("No se pudo resolver app_data_dir: {e}"))?;

    fs::create_dir_all(&data_dir).map_err(|e| {
        format!(
            "No se pudo crear el directorio de datos {:?}: {e}",
            data_dir
        )
    })?;

    Ok(data_dir.join("kitodo.db"))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let db_path = resolve_db_path(app)?;
            kitodo::integrations::github::sync::spawn_poller(db_path.clone());
            app.manage(AppState { db_path });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::tasks::init_db,
            commands::tasks::quick_add,
            commands::tasks::list_inbox,
            commands::tasks::list_today,
            commands::tasks::list_overdue,
            commands::tasks::list_upcoming,
            commands::tasks::list_project_tasks,
            commands::tasks::list_projects,
            commands::tasks::list_labels,
            commands::tasks::get_project_sort_mode,
            commands::tasks::set_project_sort_mode,
            commands::tasks::reorder_project_tasks,
            commands::tasks::reset_project_manual_order,
            commands::tasks::toggle_task,
            commands::tasks::toggle_task_with_recurrence,
            commands::tasks::update_task_title,
            commands::tasks::update_task_priority,
            commands::tasks::update_task_due_date,
            commands::tasks::update_task_recurrence,
            commands::tasks::move_task_to_project,
            commands::tasks::soft_delete_task,
            commands::tasks::restore_task,
            commands::tasks::export_backup_json,
            commands::tasks::import_backup_json,
            commands::github::github_connect,
            commands::github::github_disconnect,
            commands::github::github_list_accounts,
            commands::github::github_get_settings,
            commands::github::github_set_settings,
            commands::github::github_list_repos,
            commands::github::github_add_repo_subscription,
            commands::github::github_remove_repo_subscription,
            commands::github::github_toggle_repo_subscription,
            commands::github::github_list_repo_subscriptions,
            commands::github::github_sync_now,
            commands::github::github_get_status,
            commands::github::github_list_external_items,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn main() {
    match kitodo::cli::maybe_run_cli_from_args() {
        Ok(true) => return,
        Ok(false) => {}
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(1);
        }
    }

    run();
}
