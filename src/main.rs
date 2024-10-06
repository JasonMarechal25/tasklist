use std::env;
use std::process::ExitCode;
use std::fs;
use std::io::Write;
use serde::{Serialize, Deserialize};

fn main() -> ExitCode {
    println!("Hello, world!");
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("No command provided, goodbye.");
        return ExitCode::from(0)
    }
    let mut task_repository = TaskRepository::default();
    let param1 = &args[1];
    match param1.as_str() {
        "list" => { print_tasks(&task_repository); }
        "add" => {
            if args.len() < 3 {
                println!("Missing description to add a new task");
                return ExitCode::from(1)
            }
            task_repository.new_task(args[2].clone());
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
    description: String,
    id: i32,
}

#[derive(Clone)]
#[derive(Serialize, Deserialize)]
struct TaskRepository {
    tasks: Vec<Task>,
    last_id: i32,
}

impl Default for TaskRepository {
    fn default() -> Self {
        TaskRepository {
            tasks: vec![],
            last_id: 0,
        }
    }
}

impl TaskRepository {
    fn from_content(content: String) -> Self {
        let task_repository = TaskRepository::default();
        println!("{}", content);
        let parsed: TaskRepository = serde_json::from_str(&content).unwrap();
        parsed
    }

    fn new_task(&mut self, description: String) -> &Task {
        self.last_id += 1;
        let task = Task { description: description, id: self.last_id };
        self.tasks.push(task);
        let mut list_file = fs::File::create("task_list.txt").unwrap();
        list_file.write(
        serde_json::to_string(&self).unwrap().as_bytes());
        return self.tasks.last().unwrap();
    }

    fn tasks(&self) -> &Vec<Task> {
        return &self.tasks;
    }
}

fn print_tasks(repository: &TaskRepository) {
    for task in repository.tasks() {
        println!("{}", task.description)
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
        assert_eq!(task_repository.tasks().len(), 2);
    }

    #[test]
    fn repository_load_json() {
        let expected = vec![
            Task{id:0, description:String::from("plop")},
            Task{id:1, description:String::from("plap")}
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
        ],\
        \"last_id\": 1\
        }}\
        "));
        assert_eq!(expected, expected);
    }

}