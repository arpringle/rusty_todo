use std::collections::BTreeMap;
use std::error::Error;
use std::fs::{read_to_string, File};

use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use serde_json::to_writer_pretty;

const TASKFILE_PATH: &str = "taskfile.json";

/// A simple CLI to-do list, implemented in Rust.
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Add a new task
    Add {
        /// The description of the task
        description: String,
    },

    /// List all tasks
    List {
        #[arg(short, long, action)]
        incomplete_only: bool
    },

    /// Mark a task as done
    Done {
        /// The ID of the task
        id: usize,
    },
}

#[derive(Serialize, Deserialize)]
struct Task {
    description: String,
    finished: bool,
}

// The taskfile uses a BTreeMap with IDs as the keys.

// A Vec would probably be more fit to the purpose, but this method allows for the future
// possibility of implementing a "delete task" command where we could remove the task without
// affecting the task IDs.

// We prefer BTreeMap over HashMap in this case because a BTreeMap is ordered. This avoids us having
// to do a sort before listing the tasks.
#[derive(Serialize, Deserialize)]
struct TaskFile {
    curr_id: usize,
    tasks: BTreeMap<usize, Task>,
}

fn save_tasks(taskfile: &TaskFile) -> Result<(), Box<dyn Error>> {
    let file = File::create(TASKFILE_PATH)?;
    to_writer_pretty(file, taskfile)?;
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    // Load existing tasks or start with an empty struct
    let mut parsed_taskfile: TaskFile = match read_to_string(TASKFILE_PATH) {
        Ok(taskfile) => serde_json::from_str(&taskfile)?,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            TaskFile {
                curr_id: 1,
                tasks: BTreeMap::new(),
            }
        }
        Err(e) => return Err(Box::new(e)),
    };

    match &cli.command {
        Commands::Add { description } => {
            let task = Task {
                description: description.into(),
                finished: false,
            };

            parsed_taskfile
                .tasks
                .insert(parsed_taskfile.curr_id, task);
            parsed_taskfile.curr_id += 1;

            save_tasks(&parsed_taskfile)?;
        }

        Commands::List {incomplete_only} => {

            let mut task_found = false;

            for (id, task) in &parsed_taskfile.tasks {
                if *incomplete_only && task.finished { continue; }
                let status = if task.finished { "Complete" } else { "Incomplete" };
                println!("Task {}: {} ({})", id, task.description, status);
                task_found = true;
            }

            if !task_found { 
                println!("No tasks to display.");
                return Ok(());
            }
        }

        Commands::Done { id } => {
            let task = parsed_taskfile
                .tasks
                .get_mut(id)
                .ok_or_else(|| format!("Task with ID {} not found", id))?;

            task.finished = true;

            save_tasks(&parsed_taskfile)?
        }
    }

    Ok(())
}
