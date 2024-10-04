fn main() {
    println!("Hello, world!");
    let mut task_repository = TaskRepository::default();
    task_repository.new_task(String::from("TestTask"));
    task_repository.new_task(String::from("otherTask"));
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