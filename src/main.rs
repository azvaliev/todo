use todo_lib::{
    controller::TodoController,
    render::{term::TerminalTodoRenderer, TodoRenderer},
    store::sqlite::SqlLiteTodoStore,
};

const SQLLITE_DB: &'static str = "db.sqlite?mode=rwc";

#[tokio::main]
async fn main() {
    let todo_store = SqlLiteTodoStore::new(SQLLITE_DB)
        .await
        .expect("Failed to create todo manager");

    let todo_renderer = TerminalTodoRenderer::new();

    let mut todo_controller = TodoController::new(todo_renderer, todo_store);
    todo_controller
        .run()
        .await
        .expect("Can run todo controller");
}
