use std::env;
use std::process::ExitCode;
use std::fs;
use std::io::Write;
use std::fmt;

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

struct Task {
    description: String,
    id: i32,
}

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
    fn new_task(&mut self, description: String) -> &Task {
        self.last_id += 1;
        let task = Task { description: description, id: self.last_id };
        self.tasks.push(task);
        let mut list_file = fs::File::create("task_list.txt").unwrap();
        list_file.write(b"{\n\t\"task\":[\n");
        for task in &self.tasks {
            list_file.write(format!("\
            \t\t{{\n\
            \t\t\t\"id\": \"{id}\",\n\
            \t\t\t\"description\": \"{desc}\"\n\
            \t\t}},",
                                    id=task.id, desc=task.description).as_bytes());
        }
        list_file.write(b"\n\t}\n}");

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
}