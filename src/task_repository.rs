use serde::{Deserialize, Serialize};
use std::collections::hash_map::Values;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::fs;
use std::fs::OpenOptions;
use std::io::{BufReader, Write};
use std::path::Path;

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub enum TaskStatus {
    Todo,
    InProgress,
    Done,
}
#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: i32,
    pub description: String,
    pub status: TaskStatus,
}
#[derive(Clone, PartialEq, Debug, Default)]
pub struct TaskRepository {
    tasks: HashMap<i32, Task>,
    last_id: i32,
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

    pub fn new_task(&mut self, description: String) {
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

    pub fn delete(&mut self, id: i32) -> Option<Task> {
        self.tasks.remove(&id)
    }

    pub fn tasks(&self) -> Values<'_, i32, Task> {
        self.tasks.values()
    }

    pub fn task(&mut self, id: i32) -> &mut Task {
        self.tasks.get_mut(&id).unwrap()
    }

    pub fn task_count(&self) -> usize {
        self.tasks.len()
    }
}

pub fn load_repository(file_path: &impl AsRef<Path>) -> TaskRepository {
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

pub fn save_repository(repo: &mut TaskRepository, file_path: &impl AsRef<Path>) {
    let mut list_file = fs::File::create(file_path).unwrap();
    let _ = list_file.write(
        serde_json::to_string(&repo.serializable())
            .unwrap()
            .as_bytes(),
    );
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
    let repo = TaskRepository::from_serialization(object);
    assert_eq!(expected, repo.tasks);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn repository_dump_json() {
        let mut repo = TaskRepository::default();
        repo.new_task("Plop".to_string());
        repo.new_task("Plip".to_string());
        let dump = serde_json::to_string(&repo.serializable()).unwrap();
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
}
