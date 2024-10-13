use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fmt::{Display, Formatter};
use std::fs;
use std::fs::OpenOptions;
use std::io::{BufReader, Write};
use std::path::Path;
use std::process::ExitCode;
use std::string::ToString;
use tempfile::TempDir;

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("No command provided, goodbye.");
        return ExitCode::from(0);
    }
    println!(
        "Using repository file {}",
        &env::var("TASK_FILE").unwrap().to_string()
    );
    let mut task_repository = load_repository(&env::var("TASK_FILE").unwrap().to_string());
    let param1 = &args[1];
    match param1.as_str() {
        "list" => {
            print_tasks(&task_repository);
        }
        "add" => {
            if args.len() < 3 {
                println!("Missing description to add a new task");
                return ExitCode::from(1);
            }
            add_task(&mut task_repository, args[2].clone());
        }
        "delete" => {
            if args.len() < 3 {
                println!("Missing id of task to delete");
                return ExitCode::from(1);
            }
            delete_task(
                &mut task_repository,
                args[2].clone().parse::<i32>().unwrap(),
            );
        }
        "update" => {
            if args.len() < 4 {
                println!("Missing update parameters");
                return ExitCode::from(1);
            }
            update_task(
                &mut task_repository,
                args[2].clone().parse::<i32>().unwrap(),
                args[3].clone(),
            );
        }
        "mark-in-progress" => {
            if args.len() < 3 {
                println!("Missing id of task to progress");
                return ExitCode::from(1);
            }
            mark_in_progress(
                &mut task_repository,
                args[2].clone().parse::<i32>().unwrap(),
            );
        }
        _ => {
            println!("Unknown parameter {}", param1)
        }
    }
    ExitCode::from(0)
}

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
enum TaskStatus {
    Todo,
    InProgress,
    Done,
}

impl Display for TaskStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match *self {
            TaskStatus::Todo => write!(f, "Todo"),
            TaskStatus::InProgress => write!(f, "In Progress"),
            TaskStatus::Done => write!(f, "Done"),
        }
    }
}

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
struct Task {
    id: i32,
    description: String,
    status: TaskStatus,
}

#[derive(Clone, PartialEq, Debug, Default)]
struct TaskRepository {
    tasks: HashMap<i32, Task>,
    last_id: i32,
}

#[derive(Serialize, Deserialize)]
struct TaskRepositoryForSerialization {
    tasks: Vec<Task>,
}

impl TaskRepository {
    fn from_serialization(object: TaskRepositoryForSerialization) -> Self {
        let mut task_repository = TaskRepository::default();
        let mut max_id = 0;
        for task in object.tasks {
            if task.id > max_id {
                max_id = task.id
            }
            task_repository.tasks.insert(task.id, task);
        }
        task_repository.last_id = max_id;
        task_repository
    }

    fn new_task(&mut self, description: String) {
        self.last_id += 1;
        let task = Task {
            description,
            id: self.last_id,
            status: TaskStatus::Todo,
        };
        self.tasks.insert(self.last_id, task);
    }

    fn serializable(&self) -> TaskRepositoryForSerialization {
        let mut vec: Vec<Task> = self.tasks.values().cloned().collect();
        vec.sort_by(|a, b| a.id.cmp(&b.id));
        TaskRepositoryForSerialization { tasks: vec }
    }

    fn delete(&mut self, id: i32) -> Option<Task> {
        self.tasks.remove(&id)
    }
}

fn print_tasks(repository: &TaskRepository) {
    for task in repository.tasks.values() {
        println!("Task {}: {} {}", task.id, task.description, task.status)
    }
}

fn add_task(repo: &mut TaskRepository, desc: String) {
    repo.new_task(desc);
    save_repository(repo, &env::var("TASK_FILE").unwrap().to_string());
}

fn delete_task(repo: &mut TaskRepository, task_id: i32) -> Option<Task> {
    let ret = repo.delete(task_id);
    save_repository(repo, &env::var("TASK_FILE").unwrap().to_string());
    ret
}

fn update_task(repo: &mut TaskRepository, id: i32, new_desc: String) {
    repo.tasks.get_mut(&id).unwrap().description = new_desc;
    save_repository(repo, &env::var("TASK_FILE").unwrap().to_string());
}

fn mark_in_progress(repo: &mut TaskRepository, id: i32) {
    repo.tasks.get_mut(&id).unwrap().status = TaskStatus::InProgress;
    save_repository(repo, &env::var("TASK_FILE").unwrap().to_string());
}

fn save_repository(repo: &mut TaskRepository, file_path: &impl AsRef<Path>) {
    let mut list_file = fs::File::create(file_path).unwrap();
    let _ = list_file.write(
        serde_json::to_string(&repo.serializable())
            .unwrap()
            .as_bytes(),
    );
}

fn load_repository(file_path: &impl AsRef<Path>) -> TaskRepository {
    if !fs::exists(file_path).unwrap() {
        return TaskRepository::default();
    }
    let file = OpenOptions::new()
        .read(true)
        .create(true)
        .write(true)
        .open(file_path)
        .unwrap();
    let reader = BufReader::new(file);
    let repo_object: TaskRepositoryForSerialization = serde_json::from_reader(reader).unwrap();
    TaskRepository::from_serialization(repo_object)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    use tempfile::TempDir;

    #[test]
    fn task_added() {
        let mut task_repository = TaskRepository::default();
        let tmp_dir = TempDir::new().unwrap();
        let tmp_file = tmp_dir.path().join(Path::new("tmp_file.txt"));
        add_task(&mut task_repository, "TestTask".to_string());
        let task = &task_repository.tasks[&1];
        assert_eq!(task.description, "TestTask");
        assert_eq!(task.id, 1);
    }

    #[test]
    fn task_id_incremental() {
        let mut task_repository = TaskRepository::default();
        task_repository.new_task(String::from("TestTask"));
        task_repository.new_task(String::from("otherTask"));
        let task2 = &task_repository.tasks[&2];
        assert_eq!(task2.description, "otherTask");
        assert_eq!(task2.id, 2);
    }

    #[test]
    fn list_task() {
        let mut task_repository = TaskRepository::default();
        task_repository.new_task(String::from("TestTask"));
        task_repository.new_task(String::from("otherTask"));
        assert_eq!(task_repository.tasks.len(), 2);
    }

    #[test]
    fn repository_load_json() {
        let expected = HashMap::from([
            (
                0,
                Task {
                    id: 0,
                    description: String::from("plop"),
                    status: TaskStatus::Todo,
                },
            ),
            (
                1,
                Task {
                    id: 1,
                    description: String::from("plap"),
                    status: TaskStatus::Done,
                },
            ),
        ]);
        let content = format!(
            "\
        {{\
        \"tasks\": [\
            {{\
                \"id\": 0,\
                \"description\": \"plop\",\
                \"status\": \"Todo\"\
            }},\
            {{\
                \"id\": 1,\
                \"description\": \"plap\",\
                \"status\": \"Done\"\
            }}\
        ]\
        }}\
        "
        );
        let object: TaskRepositoryForSerialization = serde_json::from_str(&content).unwrap();
        let task_repository = TaskRepository::from_serialization(object);
        assert_eq!(expected, task_repository.tasks);
    }

    #[test]
    fn repository_dump_json() {
        let mut task_repository = TaskRepository::default();
        task_repository.new_task("Plop".to_string());
        task_repository.new_task("Plip".to_string());
        let dump = serde_json::to_string(&task_repository.serializable()).unwrap();
        assert_eq!(
            dump,
            String::from(
                "{\
        \"tasks\":[\
            {\
                \"id\":1,\
                \"description\":\"Plop\",\
                \"status\":\"Todo\"\
            },\
            {\
                \"id\":2,\
                \"description\":\"Plip\",\
                \"status\":\"Todo\"\
            }\
        ]\
        }"
            )
        )
    }

    #[test]
    fn delete_task() {
        let mut task_repository = TaskRepository::default();
        task_repository.new_task("Plop".to_string());
        task_repository.new_task("Plip".to_string());
        task_repository.delete(1);
        assert_eq!(task_repository.tasks.len(), 1);
        assert_eq!(
            task_repository.tasks[&2],
            Task {
                id: 2,
                description: String::from("Plip"),
                status: TaskStatus::Todo
            }
        );
    }

    #[test]
    fn update_task_with_desc_by_id() {
        let mut task_repository = TaskRepository::default();
        task_repository.new_task("Plop".to_string());
        task_repository.new_task("Plip".to_string());
        update_task(&mut task_repository, 2, "New desc".to_string());
        assert_eq!(task_repository.tasks[&2].description, "New desc");
    }

    #[test]
    fn update_inprogress() {
        let mut task_repository = TaskRepository::default();
        task_repository.new_task("Plop".to_string());
        mark_in_progress(&mut task_repository, 1);
        assert_eq!(task_repository.tasks[&1].status, TaskStatus::InProgress);
    }

    #[test]
    fn save_load_repo() {
        let mut task_repository = TaskRepository::default();
        task_repository.new_task("Plop".to_string());
        task_repository.new_task("Plip".to_string());
        mark_in_progress(&mut task_repository, 1);
        let tmp_dir = TempDir::new().unwrap();
        let tmp_file = tmp_dir.path().join(Path::new("tmp_file.txt"));
        save_repository(&mut task_repository, &tmp_file);
        let loaded_repo = load_repository(&tmp_file);
        assert_eq!(task_repository, loaded_repo);
    }
}
