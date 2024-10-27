use crate::task_repository::Task;
use crate::task_repository::TaskRepository;
use crate::task_repository::TaskStatus;
use std::env;
use std::process::ExitCode;
use std::string::ToString;

pub mod task_repository;

/// The main entry point of the application.
///
/// This function reads command-line arguments and the `TASK_FILE` environment variable,
/// loads the task repository, and handles the provided command.
///
/// # Returns
///
/// An `ExitCode` indicating the success or failure of the operation.
fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("No command provided, goodbye.");
        return ExitCode::from(0);
    }

    let task_file = match env::var("TASK_FILE") {
        Ok(val) => val,
        Err(_) => {
            println!("TASK_FILE environment variable not set");
            return ExitCode::from(1);
        }
    };

    println!("Reading tasks from {}", task_file);
    let mut repo = task_repository::load_repository(&task_file);

    match handle_command(&args, &mut repo) {
        Ok(_) => ExitCode::from(0),
        Err(err) => {
            println!("{}", err);
            ExitCode::from(1)
        }
    }
}

/// Handles the provided command by delegating to the appropriate function.
///
/// # Arguments
///
/// * `args` - A slice of command-line arguments.
/// * `repo` - A mutable reference to the `TaskRepository`.
///
/// # Returns
///
/// A `Result` indicating the success or failure of the operation.
fn handle_command(args: &[String], repo: &mut TaskRepository) -> Result<(), String> {
    let param1 = &args[1];
    match param1.as_str() {
        "list" => handle_list_command(args, repo),
        "add" => handle_add_command(args, repo),
        "delete" => handle_delete_command(args, repo),
        "update" => handle_update_command(args, repo),
        "mark-in-progress" => handle_mark_in_progress_command(args, repo),
        _ => Err(format!("Unknown parameter {}", param1)),
    }
}

/// Handles the "list" command to display tasks.
///
/// # Arguments
///
/// * `args` - A slice of command-line arguments.
/// * `repo` - A reference to the `TaskRepository`.
///
/// # Returns
///
/// A `Result` indicating the success or failure of the operation.
fn handle_list_command(args: &[String], repo: &TaskRepository) -> Result<(), String> {
    if args.len() == 2 {
        print_tasks(repo);
    } else if args.len() == 3 {
        match args[2].as_str() {
            "todo" => print_tasks_by_status(repo, TaskStatus::Todo),
            "done" => print_tasks_by_status(repo, TaskStatus::Done),
            "in-progress" => print_tasks_by_status(repo, TaskStatus::InProgress),
            _ => return Err("Unknown status to display".to_string()),
        }
    }
    Ok(())
}

/// Handles the "add" command to add a new task.
///
/// # Arguments
///
/// * `args` - A slice of command-line arguments.
/// * `repo` - A mutable reference to the `TaskRepository`.
///
/// # Returns
///
/// A `Result` indicating the success or failure of the operation.
fn handle_add_command(args: &[String], repo: &mut TaskRepository) -> Result<(), String> {
    if args.len() < 3 {
        return Err("Missing description to add a new task".to_string());
    }
    add_task(repo, args[2].clone());
    Ok(())
}

/// Handles the "delete" command to delete a task.
///
/// # Arguments
///
/// * `args` - A slice of command-line arguments.
/// * `repo` - A mutable reference to the `TaskRepository`.
///
/// # Returns
///
/// A `Result` indicating the success or failure of the operation.
fn handle_delete_command(args: &[String], repo: &mut TaskRepository) -> Result<(), String> {
    if args.len() < 3 {
        return Err("Missing id of task to delete".to_string());
    }
    delete_task(repo, args[2].clone().parse::<i32>().unwrap());
    Ok(())
}

/// Handles the "update" command to update a task's description.
///
/// # Arguments
///
/// * `args` - A slice of command-line arguments.
/// * `repo` - A mutable reference to the `TaskRepository`.
///
/// # Returns
///
/// A `Result` indicating the success or failure of the operation.
fn handle_update_command(args: &[String], repo: &mut TaskRepository) -> Result<(), String> {
    if args.len() < 4 {
        return Err("Missing update parameters".to_string());
    }
    update_task(
        repo,
        args[2].clone().parse::<i32>().unwrap(),
        args[3].clone(),
    );
    Ok(())
}

/// Handles the "mark-in-progress" command to mark a task as in progress.
///
/// # Arguments
///
/// * `args` - A slice of command-line arguments.
/// * `repo` - A mutable reference to the `TaskRepository`.
///
/// # Returns
///
fn handle_mark_in_progress_command(
    args: &[String],
    repo: &mut TaskRepository,
) -> Result<(), String> {
    if args.len() < 3 {
        return Err("Missing id of task to progress".to_string());
    }
    mark_in_progress(repo, args[2].clone().parse::<i32>().unwrap());
    Ok(())
}

/// Prints tasks filtered by their status.
///
/// # Arguments
///
/// * `repo` - A reference to the `TaskRepository`.
/// * `status` - The `TaskStatus` to filter tasks by.
fn print_tasks_by_status(repo: &TaskRepository, status: TaskStatus) {
    let task_list: Vec<_> = repo.tasks().filter(|task| task.status == status).collect();
    if task_list.is_empty() {
        println!("No task with status {}", status);
    } else {
        task_list.into_iter().for_each(print_task);
    }
}

/// Prints all tasks in the repository.
///
/// # Arguments
///
/// * `repository` - A reference to the `TaskRepository`.
fn print_tasks(repository: &TaskRepository) {
    if repository.task_count() > 0 {
        repository.tasks().for_each(print_task);
    } else {
        println!("Your task list is empty.");
    }
}

/// Prints a single task.
///
/// # Arguments
///
/// * `task` - A reference to the `Task` to be printed.
fn print_task(task: &Task) {
    println!(
        "Task {}: \"{}\" {}. Created at: {}. Last update: {}",
        task.id, task.description, task.status, task.created_at, task.created_at
    );
}

/// Adds a new task to the repository.
///
/// # Arguments
///
/// * `repo` - A mutable reference to the `TaskRepository`.
/// * `desc` - A string describing the new task.
fn add_task(repo: &mut TaskRepository, desc: String) {
    repo.new_task(desc);
    let var = &env::var("TASK_FILE").unwrap().to_string();
    println!("var {}", var);
    task_repository::save_repository(repo, &env::var("TASK_FILE").unwrap().to_string());
}

/// Deletes a task from the repository.
///
/// # Arguments
///
/// * `repo` - A mutable reference to the `TaskRepository`.
/// * `task_id` - The ID of the task to be deleted.
///
/// # Returns
///
/// An `Option` containing the deleted `Task` if it existed.
fn delete_task(repo: &mut TaskRepository, task_id: i32) -> Option<Task> {
    let ret = repo.delete(task_id);
    task_repository::save_repository(repo, &env::var("TASK_FILE").unwrap().to_string());
    ret
}

/// Updates the description of a task.
///
/// # Arguments
///
/// * `repo` - A mutable reference to the `TaskRepository`.
/// * `id` - The ID of the task to be updated.
/// * `new_desc` - The new description for the task.
fn update_task(repo: &mut TaskRepository, id: i32, new_desc: String) {
    let task = repo.task(id);
    task.description = new_desc;
    task_repository::save_repository(repo, &env::var("TASK_FILE").unwrap().to_string());
}

/// Marks a task as in progress.
///
/// # Arguments
///
/// * `repo` - A mutable reference to the `TaskRepository`.
/// * `id` - The ID of the task to be marked as in progress.
fn mark_in_progress(repo: &mut TaskRepository, id: i32) {
    repo.task(id).status = TaskStatus::InProgress;
    task_repository::save_repository(repo, &env::var("TASK_FILE").unwrap().to_string());
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    use tempfile::TempDir;

    #[test]
    fn task_added() {
        let mut repo = TaskRepository::default();
        let tmp_dir = TempDir::new().unwrap();
        let _ = env::set_current_dir(&tmp_dir);
        add_task(&mut repo, "TestTask".to_string());
        let task = &repo.task(1);
        assert_eq!(task.description, "TestTask");
        assert_eq!(task.id, 1);
    }

    #[test]
    fn task_id_incremental() {
        let mut repo = TaskRepository::default();
        repo.new_task(String::from("TestTask"));
        repo.new_task(String::from("otherTask"));
        let task2 = &repo.task(2);
        assert_eq!(task2.description, "otherTask");
        assert_eq!(task2.id, 2);
    }

    #[test]
    fn list_task() {
        let mut repo = TaskRepository::default();
        repo.new_task(String::from("TestTask"));
        repo.new_task(String::from("otherTask"));
        assert_eq!(repo.task_count(), 2);
    }

    #[test]
    fn delete_task() {
        let mut repo = TaskRepository::default();
        repo.new_task("Plop".to_string());
        repo.new_task("Plip".to_string());
        repo.delete(1);
        assert_eq!(repo.task_count(), 1);
        let task = repo.task(2);
        assert_eq!(task.id, 2);
        assert_eq!(task.description, String::from("Plip"));
        assert_eq!(task.status, TaskStatus::Todo);
    }

    #[test]
    fn update_task_with_desc_by_id() {
        let mut repo = TaskRepository::default();
        repo.new_task("Plop".to_string());
        repo.new_task("Plip".to_string());
        update_task(&mut repo, 2, "New desc".to_string());
        assert_eq!(repo.task(2).description, "New desc");
    }

    #[test]
    fn update_inprogress() {
        let mut repo = TaskRepository::default();
        repo.new_task("Plop".to_string());
        mark_in_progress(&mut repo, 1);
        assert_eq!(repo.task(1).status, TaskStatus::InProgress);
    }

    #[test]
    fn save_load_repo() {
        let mut repo = TaskRepository::default();
        repo.new_task("Plop".to_string());
        repo.new_task("Plip".to_string());
        mark_in_progress(&mut repo, 1);
        let tmp_dir = TempDir::new().unwrap();
        let tmp_file = tmp_dir.path().join(Path::new("tmp_file.txt"));
        task_repository::save_repository(&mut repo, &tmp_file);
        let loaded_repo = task_repository::load_repository(&tmp_file);
        assert_eq!(repo, loaded_repo);
    }
}
