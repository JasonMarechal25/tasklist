use crate::task_repository::Task;
use crate::task_repository::TaskRepository;
use crate::task_repository::TaskStatus;
use std::env;
use std::process::ExitCode;
use std::string::ToString;

pub mod task_repository;

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("No command provided, goodbye.");
        return ExitCode::from(0);
    }
    println!(
        "Reading tasks from {}",
        &env::var("TASK_FILE").unwrap().to_string()
    );
    let mut repo = task_repository::load_repository(&env::var("TASK_FILE").unwrap().to_string());
    let param1 = &args[1];
    match param1.as_str() {
        "list" => {
            if args.len() == 2 {
                print_tasks(&repo);
            } else if args.len() == 3 {
                match args[2].as_str() {
                    "todo" => {
                        for task in repo.tasks().filter(|task| task.status == TaskStatus::Todo) {
                            println!("Task {}: \"{}\" {}. Created at: {}. Last update: {}", task.id, task.description, task.status, task.created_at, task.created_at)
                        };
                    },
                    "done" => {
                        for task in repo.tasks().filter(|task| task.status == TaskStatus::Done) {
                            println!("Task {}: \"{}\" {}. Created at: {}. Last update: {}", task.id, task.description, task.status, task.created_at, task.created_at)
                        };
                    }
                    &_ => todo!()
                }
            }
        }
        "add" => {
            if args.len() < 3 {
                println!("Missing description to add a new task");
                return ExitCode::from(1);
            }
            add_task(&mut repo, args[2].clone());
        }
        "delete" => {
            if args.len() < 3 {
                println!("Missing id of task to delete");
                return ExitCode::from(1);
            }
            delete_task(&mut repo, args[2].clone().parse::<i32>().unwrap());
        }
        "update" => {
            if args.len() < 4 {
                println!("Missing update parameters");
                return ExitCode::from(1);
            }
            update_task(
                &mut repo,
                args[2].clone().parse::<i32>().unwrap(),
                args[3].clone(),
            );
        }
        "mark-in-progress" => {
            if args.len() < 3 {
                println!("Missing id of task to progress");
                return ExitCode::from(1);
            }
            mark_in_progress(&mut repo, args[2].clone().parse::<i32>().unwrap());
        }
        _ => {
            println!("Unknown parameter {}", param1)
        }
    }
    ExitCode::from(0)
}

fn print_tasks(repository: &TaskRepository) {
    for task in repository.tasks() {
        println!("Task {}: \"{}\" {}. Created at: {}. Last update: {}", task.id, task.description, task.status, task.created_at, task.created_at)
    }
}

fn add_task(repo: &mut TaskRepository, desc: String) {
    repo.new_task(desc);
    let var = &env::var("TASK_FILE").unwrap().to_string();
    println!("var {}", var);
    task_repository::save_repository(repo, &env::var("TASK_FILE").unwrap().to_string());
}

fn delete_task(repo: &mut TaskRepository, task_id: i32) -> Option<Task> {
    let ret = repo.delete(task_id);
    task_repository::save_repository(repo, &env::var("TASK_FILE").unwrap().to_string());
    ret
}

fn update_task(repo: &mut TaskRepository, id: i32, new_desc: String) {
    let task = repo.task(id);
    task.description = new_desc;
    task_repository::save_repository(repo, &env::var("TASK_FILE").unwrap().to_string());
}

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
