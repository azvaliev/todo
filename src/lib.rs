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

#[derive(Debug)]
pub enum MarkCompleteError {
    SQLXError(sqlx::Error),
    TodoNotFound,
}

impl PartialEq for MarkCompleteError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (MarkCompleteError::SQLXError(_), MarkCompleteError::SQLXError(_)) => true,
            (MarkCompleteError::TodoNotFound, MarkCompleteError::TodoNotFound) => true,
            _ => false,
        }
    }
}

#[async_trait]
pub trait TodoManager {
    /// Create a new todo
    async fn create(&mut self, todo_content: &Todo) -> Result<(), sqlx::Error>;

    /// Gets a list of todos which have not been completed, or were completed within the last 24 hours
    /// Incomplete todos should come first, ordered by recently created first
    /// Completed todos should come last, ordered by recently completed first
    async fn compile_relevant_list(&mut self) -> Result<Vec<Todo>, sqlx::Error>;

    /// Mark a todo as complete
    async fn mark_complete(&mut self, todo_id: &str) -> Result<(), MarkCompleteError>;
}
