use std::sync::Arc;

use tokio::sync::{mpsc, Mutex};

use crate::{
    render::{TodoRenderOptions, TodoRenderer, TodoRendererEvent},
    store::{MarkCompleteError, TodoStore},
    Todo,
};

pub struct TodoController<T: TodoRenderer, U: TodoStore> {
    /// Current todos state
    todos: Arc<Mutex<Vec<Todo>>>,
    /// Rendering implementation
    renderer: Arc<T>,
    /// The data store
    store: Arc<U>,
    /// Receiver for events from the UI
    rx_render_event: Arc<Mutex<mpsc::Receiver<TodoRendererEvent>>>,
}

impl<T: TodoRenderer, U: TodoStore> TodoController<T, U> {
    pub fn new(mut renderer: T, store: U) -> Self {
        // Setup a channel for listening to UI events
        let (tx_render_event, rx_render_event) = mpsc::channel::<TodoRendererEvent>(20);
        renderer.set_event_sender(tx_render_event);

        Self {
            todos: Arc::new(Mutex::new(Vec::new())),
            renderer: Arc::new(renderer),
            store: Arc::new(store),
            rx_render_event: Arc::new(Mutex::new(rx_render_event)),
        }
    }

    pub async fn run(&mut self) -> Result<(), String> {
        // Load initial todos
        {
            let mut todos = self.todos.lock().await;
            *todos = self
                .store
                .compile_relevant_list()
                .await
                .map_err(|e| format!("Failed to get todos: {}", e))?;
        }

        self.renderer
            .render(&*self.todos.lock().await, Default::default());

        let event_todos = self.todos.clone();
        let event_store = self.store.clone();
        let event_renderer = self.renderer.clone();
        let event_rx_render_event = self.rx_render_event.clone();

        // Listen for events on a separate thread
        tokio::spawn(async move {
            let mut rx_render_event = event_rx_render_event.lock().await;

            while let Some(event) = rx_render_event.recv().await {
                match event {
                    TodoRendererEvent::AddTodo(todo_content) => {
                        let mut todos = event_todos.lock().await;

                        // Create a new todo
                        let new_todo = Todo::new(todo_content);
                        let creation_result = event_store.create(&new_todo).await;

                        let error = match creation_result {
                            Ok(_) => {
                                todos.push(new_todo);
                                None
                            }
                            Err(e) => Some(format!("Failed to create todo: {}", e)),
                        };

                        event_renderer.render(&*todos, TodoRenderOptions { error });
                    }
                    TodoRendererEvent::MarkComplete(todo_id) => {
                        let mut todos = event_todos.lock().await;

                        // Mark todo as completed
                        let mark_complete_result = event_store.mark_complete(todo_id).await;

                        let error = match mark_complete_result {
                            Ok(_) => None,
                            Err(MarkCompleteError::TodoNotFound) => {
                                *todos = event_store
                                    .compile_relevant_list()
                                    .await
                                    .map_err(|e| format!("Failed to get todos: {}", e))
                                    .unwrap();

                                Some(format!("This todo does not exist"))
                            }
                            Err(e) => Some(format!("Failed to mark todo as complete: {}", e)),
                        };

                        event_renderer.render(&*todos, TodoRenderOptions { error });
                    }
                    _ => {
                        todo!()
                    }
                }
            }
        })
        .await
        .expect("Failed to spawn event listener");

        Ok(())
    }
}
