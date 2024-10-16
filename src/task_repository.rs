use serde::{Deserialize, Serialize};
use std::collections::hash_map::Values;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::fs;
use std::fs::OpenOptions;
use std::io::{BufReader, Write};
use std::path::Path;
use chrono::{DateTime, Local, TimeZone};

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
    pub created_at: DateTime<Local>,
    pub updated_at: DateTime<Local>,
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
            created_at: Local::now(),
            updated_at: Local::now(),
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn repository_load_json() {
        let mut expected = HashMap::from([
            (
                0,
                Task {
                    id: 0,
                    description: String::from("plop"),
                    status: TaskStatus::Todo,
                    created_at: DateTime::from(Local.with_ymd_and_hms(2024, 01, 01, 01, 02, 03).unwrap()),
                    updated_at: DateTime::from(Local.with_ymd_and_hms(2024, 02, 01, 05, 02, 03).unwrap()),
                },
            ),
            (
                1,
                Task {
                    id: 1,
                    description: String::from("plap"),
                    status: TaskStatus::Done,
                    created_at: DateTime::from(Local.with_ymd_and_hms(2024, 03, 06, 01, 02, 03).unwrap()),
                    updated_at: DateTime::from(Local.with_ymd_and_hms(2024, 02, 01, 05, 12, 03).unwrap()),
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
                \"status\": \"Todo\",\
                \"created_at\":\"2024-10-16T14:45:18.529270461+02:00\",\
                \"updated_at\":\"2024-10-16T14:45:18.529569668+02:00\"
            }},\
            {{\
                \"id\": 1,\
                \"description\": \"plap\",\
                \"status\": \"Done\",\
                \"created_at\":\"2024-10-16T14:45:18.529270461+02:00\",\
                \"updated_at\":\"2024-10-16T14:45:18.529569668+02:00\"
            }}\
        ]\
        }}\
        "
        );
        let object: TaskRepositoryForSerialization = serde_json::from_str(&content).unwrap();
        let repo = TaskRepository::from_serialization(object);

        //Can't assert_eq because repo because of dates
        assert_eq!(repo.tasks.len(), expected.len());
        for (key, value) in repo.tasks.iter() {
            assert_eq!(value.id, expected[key].id);
            assert_eq!(value.description, expected[key].description);
            assert_eq!(value.status, expected[key].status);
        }
    }
}
