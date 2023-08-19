pub mod sqlite;

use async_trait::async_trait;
use sqlx::FromRow;
use std::time::{SystemTime, UNIX_EPOCH};

use cuid2::cuid;

#[derive(Debug, FromRow)]
pub struct Todo {
    pub id: String,
    pub content: String,
    /// Unix timestamp
    pub created_at: i64,
    /// Unix timestamp
    pub completed_at: Option<i64>,
}

impl Todo {
    pub fn new(content: String) -> Todo {
        let created_at = Todo::get_timestamp_now();

        Todo {
            id: cuid(),
            content,
            created_at,
            completed_at: None,
        }
    }

    pub fn get_timestamp_now() -> i64 {
        let timestamp_now: i64 = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs()
            .try_into()
            .expect("Unix timestamp overflowed");

        timestamp_now
    }
}

#[async_trait]
pub trait TodoManager {
    /// Create a new todo
    async fn create(&mut self, todo_content: &Todo) -> Result<(), String>;

    /// Gets a list of todos which have not been completed, or were completed within the last 24 hours
    async fn compile_relevant_list(&mut self) -> Result<Vec<Todo>, String>;

    /// Mark a todo as complete
    async fn mark_complete(&mut self, todo_id: &str) -> Result<(), String>;
}

