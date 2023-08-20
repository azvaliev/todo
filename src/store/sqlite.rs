use std::sync::Arc;

use crate::{store::MarkCompleteError, Todo};
use async_trait::async_trait;
use sqlx::SqlitePool;

use super::TodoStore;

#[derive(Clone)]
pub struct SqlLiteTodoStore {
    db: Arc<SqlitePool>,
}

impl SqlLiteTodoStore {
    /// Create a new SqlLiteTodoManager
    /// Takes a path to the sqlite database
    pub async fn new(sqlite_path: &str) -> Result<SqlLiteTodoStore, String> {
        // Connect to the sqlite database
        let db = SqlitePool::connect(sqlite_path)
            .await
            .map_err(|e| format!("Failed to connect to sqlite database: {}", e))?;

        Ok(SqlLiteTodoStore { db: Arc::new(db) })
    }

    pub async fn migrate(&self) -> Result<(), String> {
        sqlx::migrate!()
            .run(&*self.db)
            .await
            .map_err(|e| format!("Failed to migrate database: {}", e))?;

        Ok(())
    }
}

#[async_trait]
impl TodoStore for SqlLiteTodoStore {
    async fn create(&self, todo: &Todo) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "
            INSERT INTO todos (id, content, created_at, completed_at)
            VALUES (?, ?, ?, ?)
            ",
            todo.id,
            todo.content,
            todo.created_at,
            todo.completed_at,
        )
        .execute(&*self.db)
        .await?;

        Ok(())
    }

    async fn compile_relevant_list(&self) -> Result<Vec<Todo>, sqlx::Error> {
        let timestamp_now = Todo::get_timestamp_now();

        let timestamp_24_hours_ago = timestamp_now - (24 * 60 * 60);

        let mut relevant_todos: Vec<Todo> = Vec::new();
        let mut incomplete_todos = sqlx::query_as!(
            Todo,
            "
            SELECT id, content, created_at, completed_at
            FROM todos
            WHERE completed_at IS NULL
            ORDER BY created_at DESC
            ",
        )
        .fetch_all(&*self.db)
        .await?;

        let mut completed_todos = sqlx::query_as!(
            Todo,
            "
            SELECT id, content, created_at, completed_at
            FROM todos
            WHERE completed_at > ?
            ORDER BY completed_at DESC
            ",
            timestamp_24_hours_ago,
        )
        .fetch_all(&*self.db)
        .await?;

        relevant_todos.append(&mut incomplete_todos);
        relevant_todos.append(&mut completed_todos);

        Ok(relevant_todos)
    }

    async fn mark_complete(&self, todo_id: String) -> Result<(), MarkCompleteError> {
        let timestamp_now = Todo::get_timestamp_now();

        let affected_count = sqlx::query!(
            "
           UPDATE Todos
           SET completed_at = ?
           WHERE id = ?
           ",
            timestamp_now,
            todo_id,
        )
        .execute(&*self.db)
        .await
        .map_err(|err| MarkCompleteError::SQLXError(err))?;

        if affected_count.rows_affected() != 1 {
            return Err(MarkCompleteError::TodoNotFound);
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use std::thread;

    use crate::{
        store::{MarkCompleteError, TodoStore},
        Todo,
    };

    use super::SqlLiteTodoStore;

    async fn create_in_memory_sqlite_todo_manager() -> SqlLiteTodoStore {
        let todo_manager = SqlLiteTodoStore::new(":memory:")
            .await
            .expect("Failed to create todo manager");

        todo_manager
            .migrate()
            .await
            .expect("Failed to migrate database");

        todo_manager
    }

    async fn create_todo(
        todo_manager: &mut SqlLiteTodoStore,
        todo_content: &str,
    ) -> (Todo, Result<(), sqlx::Error>) {
        let todo = Todo::new(String::from(todo_content));
        let todo_create_result = todo_manager.create(&todo).await;

        (todo, todo_create_result)
    }

    #[tokio::test]
    async fn create_todo_success() {
        let mut todo_manager = create_in_memory_sqlite_todo_manager().await;

        let todo_content = String::from("Test");
        let (todo, todo_create_result) = create_todo(&mut todo_manager, &todo_content).await;

        assert!(todo_create_result.is_ok());

        let todos = todo_manager
            .compile_relevant_list()
            .await
            .expect("Failed to compile relevant list");

        assert_eq!(todos.len(), 1);
        assert_eq!(todos[0].id, todo.id);
        assert_eq!(todos[0].content, todo_content);
        assert_eq!(todos[0].created_at, todo.created_at);
        assert_eq!(todos[0].completed_at, None);
    }

    #[tokio::test]
    async fn mark_todo_complete() {
        let mut todo_manager = create_in_memory_sqlite_todo_manager().await;

        let todo_title = String::from("Test");
        let (todo, todo_create_result) = create_todo(&mut todo_manager, &todo_title).await;

        assert!(todo_create_result.is_ok());

        todo_manager
            .mark_complete(todo.id)
            .await
            .expect("Failed to mark todo as complete");

        let todos = todo_manager
            .compile_relevant_list()
            .await
            .expect("Failed to compile relevant list");

        assert_eq!(todos.len(), 1);
        assert_eq!(todos[0].content, todo_title);
        assert!(todos[0].completed_at.is_some());
    }

    #[tokio::test]
    async fn mark_nonexistent_todo_complete() {
        let todo_manager = create_in_memory_sqlite_todo_manager().await;

        let mark_complete_result = todo_manager
            .mark_complete(String::from("nonexistent"))
            .await;

        assert!(mark_complete_result.is_err());
        assert_eq!(
            mark_complete_result.unwrap_err(),
            MarkCompleteError::TodoNotFound
        );
    }

    #[tokio::test]
    /// Incomplete todos should come first, ordered by recently created first
    /// Completed todos should come last, ordered by recently completed first
    async fn get_todos_ordering() {
        let mut todo_manager = create_in_memory_sqlite_todo_manager().await;

        let first_incomplete_todo_content = String::from("Test");
        let first_completed_todo_content = String::from("Test 2");
        let last_incomplete_todo_content = String::from("Test 3");
        let last_completed_todo_content = String::from("Test 4");

        let (first_incomplete_todo, _) =
            create_todo(&mut todo_manager, &first_incomplete_todo_content).await;
        // Sleep to ensure that the created_at timestamps are different
        thread::sleep(std::time::Duration::from_secs(1));
        let (last_incomplete_todo, _) =
            create_todo(&mut todo_manager, &last_incomplete_todo_content).await;

        let (first_complete_todo, _) =
            create_todo(&mut todo_manager, &first_completed_todo_content).await;
        let (last_complete_todo, _) =
            create_todo(&mut todo_manager, &last_completed_todo_content).await;

        todo_manager
            .mark_complete(first_complete_todo.id.clone())
            .await
            .expect("Failed to mark todo as complete");
        // Sleep to ensure that the completed_at timestamps are different
        thread::sleep(std::time::Duration::from_secs(1));
        todo_manager
            .mark_complete(last_complete_todo.id.clone())
            .await
            .expect("Failed to mark todo as complete");

        let todos = todo_manager
            .compile_relevant_list()
            .await
            .expect("Failed to compile relevant list");

        dbg!(&todos);

        assert_eq!(todos.len(), 4);
        assert_eq!(todos[1].id, first_incomplete_todo.id);
        assert_eq!(todos[0].id, last_incomplete_todo.id);
        assert_eq!(todos[2].id, last_complete_todo.id);
        assert_eq!(todos[3].id, first_complete_todo.id);
    }
}
