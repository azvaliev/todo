use tokio::sync::mpsc;

use crate::Todo;

pub mod term;

pub enum TodoRendererEvent {
    MarkComplete(String),
    AddTodo(String),
    Exit,
}

pub struct TodoRenderOptions {
    pub error: Option<String>,
}

impl Default for TodoRenderOptions {
    fn default() -> Self {
        Self { error: None }
    }
}

pub trait TodoRenderer: Send + Sync + 'static {
    fn new() -> Self;
    fn render(&mut self, todos: &Vec<Todo>, options: TodoRenderOptions);
    fn set_event_sender(&mut self, event_sender: mpsc::Sender<TodoRendererEvent>);
}
