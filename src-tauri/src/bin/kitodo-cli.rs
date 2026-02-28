use std::path::PathBuf;

use clap::{Parser, Subcommand};
use kitodo::db;
use serde::Serialize;

#[derive(Parser)]
#[command(name = "kitodo-cli")]
#[command(about = "Minimal CLI for KitoDo", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Today {
        #[arg(long, default_value_t = false)]
        all: bool,
        #[arg(long, default_value_t = false)]
        json: bool,
    },
    Overdue {
        #[arg(long, default_value_t = false)]
        all: bool,
        #[arg(long, default_value_t = false)]
        json: bool,
    },
    Inbox {
        #[arg(long, default_value_t = false)]
        all: bool,
        #[arg(long, default_value_t = false)]
        json: bool,
    },
    Add {
        input: String,
        #[arg(long, default_value_t = false)]
        json: bool,
    },
}

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

fn main() -> Result<(), String> {
    let cli = Cli::parse();
    let db_path = resolve_db_path()?;
    db::repo::init_db(&db_path).map_err(|e| e.to_string())?;

    match cli.command {
        Commands::Today { all, json } => {
            let tasks = db::repo::list_today(&db_path, all).map_err(|e| e.to_string())?;
            print_tasks(tasks, json)
        }
        Commands::Overdue { all, json } => {
            let tasks = db::repo::list_overdue(&db_path, all).map_err(|e| e.to_string())?;
            print_tasks(tasks, json)
        }
        Commands::Inbox { all, json } => {
            let tasks = db::repo::list_inbox(&db_path, all).map_err(|e| e.to_string())?;
            print_tasks(tasks, json)
        }
        Commands::Add { input, json } => {
            let task = db::repo::quick_add(&db_path, &input).map_err(|e| e.to_string())?;
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&TaskOutput { task }).map_err(|e| e.to_string())?
                );
            } else {
                println!("OK");
            }
            Ok(())
        }
    }
}
