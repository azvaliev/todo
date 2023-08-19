use std::time::{UNIX_EPOCH, SystemTime};

use async_trait::async_trait;
use crate::{TodoManager,Todo};
use sqlx::{sqlite::SqliteConnection, Connection};

pub struct SqlLiteTodoManager {
    db: SqliteConnection,
}

impl SqlLiteTodoManager {
    /// Create a new SqlLiteTodoManager
    /// Takes a path to the sqlite database
    pub async fn new(
        sqlite_path: &str,
    ) -> Result<SqlLiteTodoManager, String> {
        // Connect to the sqlite database
        let db = SqliteConnection::connect(sqlite_path)
            .await
            .map_err(|e| format!("Failed to connect to sqlite database: {}", e))?;

        Ok(SqlLiteTodoManager { db })
    }

    pub async fn migrate(&mut self) -> Result<(), String> {
        sqlx::migrate!()
            .run(&mut self.db)
            .await
            .map_err(|e| format!("Failed to migrate database: {}", e))?;
            
        Ok(())
    }

}

#[async_trait]
impl TodoManager for SqlLiteTodoManager {
    async fn create(&mut self, todo: &Todo) -> Result<(), String> {
        sqlx::query!(
            "
            INSERT INTO todos (id, content, created_at, completed_at)
            VALUES (?, ?, ?, ?)
            ",
            todo.id,
            todo.content,
            todo.created_at,
            todo.completed_at,
        ).fetch_one(&mut self.db)
            .await
            .map_err(|err| format!("Failed to create todo: {}", err))?;

        Ok(())
    }

    async fn compile_relevant_list(&mut self) -> Result<Vec<Todo>, String> {
        let timestamp_now = Todo::get_timestamp_now();

        let timestamp_24_hours_ago = timestamp_now - (24 * 60 * 60);

        let relevant_todos = sqlx::query_as!(
            Todo,
            "
            SELECT id, content, created_at, completed_at
            FROM todos
            WHERE completed_at IS NULL OR completed_at > ?
            ORDER BY created_at DESC
            ",
            timestamp_24_hours_ago,
        ).fetch_all(&mut self.db).await;

        match relevant_todos {
            Ok(todos) => Ok(todos),
            Err(e) => Err(format!("Failed to compile relevant list: {}", e)),
        }
    }

    async fn mark_complete(&mut self, todo_id: &str) -> Result<(), String> {
        let timestamp_now = Todo::get_timestamp_now();

       sqlx::query!(
           "
           UPDATE Todos
           SET completed_at = ?
           WHERE id = ?
           ",
           timestamp_now,
           todo_id,
        ).fetch_one(&mut self.db)
           .await
           .map_err(|err| format!("Failed to mark todo as complete: {}", err))?;

       Ok(())
    }

}
