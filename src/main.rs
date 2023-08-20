use todo_lib::{
    store::{sqlite::SqlLiteTodoStore, TodoStore},
    Todo,
};

const SQLLITE_DB: &'static str = "db.sqlite?mode=rwc";

#[tokio::main]
async fn main() {
    println!("Hello, world!");

    let todo_manager = SqlLiteTodoStore::new(SQLLITE_DB)
        .await
        .expect("Failed to create todo manager");

    todo_manager
        .migrate()
        .await
        .expect("Failed to migrate database");

    let _ = todo_manager.create(&Todo::new(String::from("Test"))).await;
    let todos = todo_manager.compile_relevant_list().await;
    let _ = dbg!(todos);
}
