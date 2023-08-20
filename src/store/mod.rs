use async_trait::async_trait;

use crate::Todo;

pub mod sqlite;

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
