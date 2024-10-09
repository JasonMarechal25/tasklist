use std::collections::HashMap;
use std::env;
use std::process::ExitCode;
use std::fs;
use std::io::{BufReader, Write};
use serde::{Serialize, Deserialize};

fn main() -> ExitCode {
    println!("Hello, world!");
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("No command provided, goodbye.");
        return ExitCode::from(0);
    }
    let file = fs::File::open("task_list.txt").unwrap();
    let reader = BufReader::new(file);
    let repo_object: TaskRepositoryForSerialization = serde_json::from_reader(reader).unwrap();
    let mut task_repository = TaskRepository::from_serialization(repo_object);
    print_tasks(&task_repository);
    let param1 = &args[1];
    match param1.as_str() {
        "list" => { print_tasks(&task_repository); }
        "add" => {
            if args.len() < 3 {
                println!("Missing description to add a new task");
                return ExitCode::from(1);
            }
            task_repository.new_task(args[2].clone());
        }
        "delete" => {
            if args.len() < 3 {
                println!("Missing id of task to delete");
                return ExitCode::from(1);
            }
            task_repository.delete(args[2].clone().parse::<i32>().unwrap());
        }
        _ => { println!("Unknown parameter {}", param1) }
    }
    print_tasks(&task_repository);
    ExitCode::from(0)
}

#[derive(PartialEq)]
#[derive(Debug)]
#[derive(Clone)]
#[derive(Serialize, Deserialize)]
struct Task {
    id: i32,
    description: String,
}

#[derive(Clone)]
struct TaskRepository {
    tasks: HashMap<i32, Task>,
    last_id: i32,
}

#[derive(Serialize, Deserialize)]
struct TaskRepositoryForSerialization {
    tasks: Vec<Task>,
}

impl Default for TaskRepository {
    fn default() -> Self {
        TaskRepository {
            tasks: HashMap::new(),
            last_id: 0,
        }
    }
}

impl TaskRepository {
    fn from_content(content: String) -> Self {
        let task_repository = TaskRepository::default();
        let object: TaskRepositoryForSerialization = serde_json::from_str(&content).unwrap();
        Self::from_serialization(object)
    }

    fn from_serialization(object: TaskRepositoryForSerialization) -> Self {
        let mut task_repository = TaskRepository::default();
        let mut max_id = 0;
        for task in object.tasks {
            if task.id > max_id { max_id = task.id }
            task_repository.tasks.insert(task.id, task);
        }
        task_repository.last_id = max_id;
        task_repository
    }

    fn new_task(&mut self, description: String) -> &Task {
        self.last_id += 1;
        let task = Task { description: description, id: self.last_id };
        self.tasks.insert(self.last_id, task);
        let mut list_file = fs::File::create("task_list.txt").unwrap();
        let _ = list_file.write(
            serde_json::to_string(&self.serializable()).unwrap().as_bytes());
        &self.tasks[&self.last_id]
    }

    fn serializable(&self) -> TaskRepositoryForSerialization {
        let mut vec: Vec<Task> = self.tasks.values().cloned().collect();
        vec.sort_by(|a, b| a.id.cmp(&b.id));
        TaskRepositoryForSerialization { tasks: vec }
    }

    fn delete(&mut self, id: i32) -> Option<Task> {
        let ret = self.tasks.remove(&id);
        let mut list_file = fs::File::create("task_list.txt").unwrap();
        let _ = list_file.write(
            serde_json::to_string(&self.serializable()).unwrap().as_bytes());
        ret
    }
}

fn print_tasks(repository: &TaskRepository) {
    for (id, task) in &repository.tasks {
        println!("Task {}: {}", task.id, task.description)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn task_added() {
        let mut task_repository = TaskRepository::default();
        let task = task_repository.new_task(String::from("TestTask"));
        assert_eq!(task.description, "TestTask");
        assert_eq!(task.id, 1);
    }

    #[test]
    fn task_id_incremental() {
        let mut task_repository = TaskRepository::default();
        task_repository.new_task(String::from("TestTask"));
        let task2 = task_repository.new_task(String::from("otherTask"));
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
        let expected = vec![
            Task { id: 0, description: String::from("plop") },
            Task { id: 1, description: String::from("plap") }
        ];
        let task_repository = TaskRepository::from_content(format!("\
        {{\
        \"tasks\": [\
            {{\
                \"id\": 0,\
                \"description\": \"plop\"\
            }},\
            {{\
                \"id\": 1,\
                \"description\": \"plap\"\
            }}\
        ]\
        }}\
        "));
        assert_eq!(expected, expected);
    }

    #[test]
    fn repository_dump_json() {
        let mut task_repository = TaskRepository::default();
        task_repository.new_task("Plop".to_string());
        task_repository.new_task("Plip".to_string());
        let dump = serde_json::to_string(&task_repository.serializable()).unwrap();
        assert_eq!(dump, String::from("{\
        \"tasks\":[\
            {\
                \"id\":1,\
                \"description\":\"Plop\"\
            },\
            {\
                \"id\":2,\
                \"description\":\"Plip\"\
            }\
        ]\
        }"))
    }

    #[test]
    fn delete_task() {
        let mut task_repository = TaskRepository::default();
        task_repository.new_task("Plop".to_string());
        task_repository.new_task("Plip".to_string());
        task_repository.delete(1);
        assert_eq!(task_repository.tasks.len(), 1);
        assert_eq!(task_repository.tasks[&2], Task { id: 2, description: String::from("Plip") });
    }
}