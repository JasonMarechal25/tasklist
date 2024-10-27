use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::collections::hash_map::Values;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::fs;
use std::fs::OpenOptions;
use std::io::{BufReader, Write};
use std::path::Path;

/// Represents the status of a task.
#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub enum TaskStatus {
    Todo,
    InProgress,
    Done,
}

/// Represents a task with an ID, description, status, and timestamps.
#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: i32,
    pub description: String,
    pub status: TaskStatus,
    pub created_at: DateTime<Local>,
    pub updated_at: DateTime<Local>,
}

/// A repository for managing tasks, including a map of tasks and the last assigned ID.
#[derive(Clone, PartialEq, Debug, Default)]
pub struct TaskRepository {
    tasks: HashMap<i32, Task>,
    last_id: i32,
}

impl Display for TaskStatus {
    /// Formats the `TaskStatus` for display.
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match *self {
            TaskStatus::Todo => write!(f, "Todo"),
            TaskStatus::InProgress => write!(f, "In Progress"),
            TaskStatus::Done => write!(f, "Done"),
        }
    }
}

/// A struct used for serializing and deserializing `TaskRepository`
/// In `TaskRepository` `Task`s objects are stored in a hashmap
/// Serializing a hash map in json produce a map <id,task>
/// Since each task already hold its id we prefere to store vec instead
/// The json produced is lighter and more readable
#[derive(Serialize, Deserialize)]
struct TaskRepositoryForSerialization {
    tasks: Vec<Task>,
}

impl TaskRepository {
    /// Creates a `TaskRepository` from a `TaskRepositoryForSerialization` object.
    ///
    /// # Arguments
    ///
    /// * `object` - A `TaskRepositoryForSerialization` object.
    ///
    /// # Returns
    ///
    /// A `TaskRepository` instance.
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

    /// Adds a new task with the given description to the repository.
    ///
    /// # Arguments
    ///
    /// * `description` - A string describing the task.
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

    /// Converts the `TaskRepository` into a `TaskRepositoryForSerialization` object.
    ///
    /// # Returns
    ///
    /// A `TaskRepositoryForSerialization` object.
    fn serializable(&self) -> TaskRepositoryForSerialization {
        let mut vec: Vec<Task> = self.tasks.values().cloned().collect();
        vec.sort_by(|a, b| a.id.cmp(&b.id));
        TaskRepositoryForSerialization { tasks: vec }
    }

    /// Deletes a task with the given ID from the repository.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the task to delete.
    ///
    /// # Returns
    ///
    /// An `Option` containing the deleted task if it existed.
    pub fn delete(&mut self, id: i32) -> Option<Task> {
        self.tasks.remove(&id)
    }

    /// Returns an iterator over the tasks in the repository.
    ///
    /// # Returns
    ///
    /// An iterator over the tasks.
    pub fn tasks(&self) -> Values<'_, i32, Task> {
        self.tasks.values()
    }

    /// Returns a mutable reference to the task with the given ID.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the task to retrieve.
    ///
    /// # Returns
    ///
    /// A mutable reference to the task.
    pub fn task(&mut self, id: i32) -> &mut Task {
        self.tasks.get_mut(&id).unwrap()
    }

    /// Returns the number of tasks in the repository.
    ///
    /// # Returns
    ///
    /// The number of tasks.
    pub fn task_count(&self) -> usize {
        self.tasks.len()
    }
}

/// Load a `TaskRepository` from a JSON file at the provided path.
///
/// If the file does not exist, a default `TaskRepository` is returned.
///
/// # Arguments
///
/// * `file_path` - A reference to a path that implements the `AsRef<Path>` trait.
///
/// # Returns
///
/// A `TaskRepository` loaded from the JSON file.
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

/// Save a `TaskRepository` to a JSON file at the provided path.
///
/// # Arguments
///
/// * `repo` - A mutable reference to the `TaskRepository` to be saved.
/// * `file_path` - A reference to a path that implements the `AsRef<Path>` trait.
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
    use chrono::TimeZone;
    use serde_json::Value;

    #[test]
    fn repository_save_json() {
        let mut repo = TaskRepository::default();
        repo.new_task(String::from("plop"));
        repo.new_task(String::from("plap"));
        repo.task(1).status = TaskStatus::Done;
        let serialized_data = serde_json::to_string(&repo.serializable()).unwrap();
        let json_object: Value = serde_json::from_str(&serialized_data).unwrap();

        assert_eq!(json_object["tasks"][0]["description"], "plop");
        assert_eq!(json_object["tasks"][0]["status"], "Done");
        assert_eq!(json_object["tasks"][1]["description"], "plap");
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
                    created_at: DateTime::from(
                        Local.with_ymd_and_hms(2024, 01, 01, 01, 02, 03).unwrap(),
                    ),
                    updated_at: DateTime::from(
                        Local.with_ymd_and_hms(2024, 02, 01, 05, 02, 03).unwrap(),
                    ),
                },
            ),
            (
                1,
                Task {
                    id: 1,
                    description: String::from("plap"),
                    status: TaskStatus::Done,
                    created_at: DateTime::from(
                        Local.with_ymd_and_hms(2024, 03, 06, 01, 02, 03).unwrap(),
                    ),
                    updated_at: DateTime::from(
                        Local.with_ymd_and_hms(2024, 02, 01, 05, 12, 03).unwrap(),
                    ),
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
