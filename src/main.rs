use std::cell::Cell;

fn main() {
    println!("Hello, world!");
}

struct Task {
    description: String,
    id: i32,
}

struct TaskRepository {
    next_id: Cell<i32>,
}

impl Default for TaskRepository {
    fn default() -> Self {
        TaskRepository { next_id: Cell::new(1), }
    }
}

impl TaskRepository {
    fn new_task(&self, description: String) -> Task {
        return Task { description: description, id: self.next_id.replace(self.next_id.get()+1) };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn task_added() {
        let task_repository = TaskRepository::default();
        let task = task_repository.new_task(String::from("TestTask"));
        assert_eq!(task.description, "TestTask");
        assert_eq!(task.id, 1);
    }

    #[test]
    fn task_id_incremental() {
        let task_repository = TaskRepository::default();
        let task = task_repository.new_task(String::from("TestTask"));
        let task2 = task_repository.new_task(String::from("otherTask"));
        assert_eq!(task2.description, "otherTask");
        assert_eq!(task2.id, 2);
    }
}