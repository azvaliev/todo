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
        let unix_timestamp: i64 = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs()
            .try_into()
            .expect("Unix timestamp overflowed");

        Todo {
            id: cuid(),
            content,
            created_at: unix_timestamp,
            completed_at: None,
        }
    }
}

#[async_trait]
pub trait TodoManager {
    /// Create a new todo
    async fn create<'a>(&mut self, todo_content: &'a Todo) -> Result<&'a Todo, String>;

    /// Gets a list of todos which have not been completed, or were completed within the last 24 hours
    async fn compile_relevant_list(&mut self) -> Result<Vec<Todo>, String>;

    /// Mark a todo as complete
    async fn mark_complete(&mut self, todo_id: &str) -> Result<Todo, String>;
}

