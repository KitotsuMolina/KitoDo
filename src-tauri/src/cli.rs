use std::path::PathBuf;

use serde::Serialize;

use crate::db;

#[derive(Serialize)]
struct TasksOutput {
    tasks: Vec<db::models::TaskDTO>,
}

#[derive(Serialize)]
struct TaskOutput {
    task: db::models::TaskDTO,
}

fn resolve_db_path() -> Result<PathBuf, String> {
    let mut dir = dirs::data_dir().ok_or_else(|| "No se pudo resolver data_dir".to_string())?;
    dir.push("kitodo");
    std::fs::create_dir_all(&dir)
        .map_err(|e| format!("No se pudo crear directorio de datos {:?}: {e}", dir))?;
    Ok(dir.join("kitodo.db"))
}

fn print_tasks(tasks: Vec<db::models::TaskDTO>, as_json: bool) -> Result<(), String> {
    if as_json {
        println!(
            "{}",
            serde_json::to_string_pretty(&TasksOutput { tasks }).map_err(|e| e.to_string())?
        );
    } else {
        for task in tasks {
            println!(
                "[{}] {} {}{}",
                task.priority,
                task.title,
                task.due_date
                    .as_deref()
                    .map(|d| format!("(due {d}) "))
                    .unwrap_or_default(),
                task.project_name
                    .as_deref()
                    .map(|p| format!("@{p}"))
                    .unwrap_or_default()
            );
        }
    }
    Ok(())
}

pub fn maybe_run_cli_from_args() -> Result<bool, String> {
    let args = std::env::args().collect::<Vec<_>>();
    if args.len() <= 1 {
        return Ok(false);
    }

    let command = args[1].as_str();
    let is_cli_cmd = matches!(command, "today" | "overdue" | "inbox" | "add");
    if !is_cli_cmd {
        return Ok(false);
    }

    let all = args.iter().any(|a| a == "--all");
    let json = args.iter().any(|a| a == "--json");

    let db_path = resolve_db_path()?;
    db::repo::init_db(&db_path).map_err(|e| e.to_string())?;

    match command {
        "today" => {
            let tasks = db::repo::list_today(&db_path, all).map_err(|e| e.to_string())?;
            print_tasks(tasks, json)?;
        }
        "overdue" => {
            let tasks = db::repo::list_overdue(&db_path, all).map_err(|e| e.to_string())?;
            print_tasks(tasks, json)?;
        }
        "inbox" => {
            let tasks = db::repo::list_inbox(&db_path, all).map_err(|e| e.to_string())?;
            print_tasks(tasks, json)?;
        }
        "add" => {
            if args.len() < 3 {
                return Err("Uso: kitodo add \"texto\" [--json]".to_string());
            }

            let input = args[2].clone();
            let task = db::repo::quick_add(&db_path, &input).map_err(|e| e.to_string())?;
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&TaskOutput { task }).map_err(|e| e.to_string())?
                );
            } else {
                println!("OK");
            }
        }
        _ => {}
    }

    Ok(true)
}
