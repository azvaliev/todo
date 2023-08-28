use crossterm::{
    cursor, queue,
    style::{self, Print, PrintStyledContent, Stylize},
    terminal,
};
use std::{
    cmp::min,
    io::{stdout, Stdout, Write},
};

use tokio::sync::mpsc;

use crate::Todo;

use super::{TodoRenderOptions, TodoRenderer, TodoRendererEvent};

pub struct TerminalTodoRenderer {
    stdout: Stdout,
    event_sender: Option<mpsc::Sender<TodoRendererEvent>>,
}

const HEADER_LINES: u16 = 2;
const FOOTER_LINES: u16 = 2;
const ERROR_LINES: u16 = 1;
static INSTRUCTIONS: &str = "Press 'a' to add todo, 'space' to toggle todo, 'q' to quit";

impl TerminalTodoRenderer {
    fn format_todo_line(todo: &Todo) -> PrintStyledContent<String> {
        let todo_line = match todo.completed_at {
            Some(_) => {
                let raw_todo_line = format!("[x] {}\n", todo.content);
                style::PrintStyledContent(raw_todo_line.white().dim().crossed_out())
            }
            None => {
                let raw_todo_line = format!("[ ] {}\n", todo.content);
                style::PrintStyledContent(raw_todo_line.white())
            }
        };

        todo_line
    }
}

impl TodoRenderer for TerminalTodoRenderer {
    fn new() -> Self {
        Self {
            stdout: stdout(),
            event_sender: None,
        }
    }

    fn render(&mut self, todos: &Vec<Todo>, options: TodoRenderOptions) {
        // Calculating dimensions
        let (_col_count, row_count) = terminal::size().expect("Can get terminal size");

        let body_size: usize = {
            let body_size_without_error = row_count - HEADER_LINES - FOOTER_LINES - 1;

            if options.error.is_some() {
                (body_size_without_error - ERROR_LINES).into()
            } else {
                body_size_without_error.into()
            }
        };
        let non_empty_body_size = min(body_size, todos.len());
        let empty_body_size = body_size - non_empty_body_size;

        // Clear screen
        queue!(
            self.stdout,
            terminal::Clear(terminal::ClearType::All),
            cursor::MoveTo(0, 0)
        )
        .expect("Can print to stdout and move cursor");

        // Enable raw mode since we are manually queueing and flushing
        terminal::enable_raw_mode().expect("Can enable raw mode");

        // App header
        queue!(self.stdout, style::PrintStyledContent("Todos\n\n".bold()))
            .expect("Can print to stdout");

        // App body
        for todo in todos[0..non_empty_body_size].iter() {
            let formatted_todo = TerminalTodoRenderer::format_todo_line(todo);
            queue!(self.stdout, cursor::MoveToColumn(0), formatted_todo,)
                .expect("Can print to stdout");
        }
        for _ in 0..empty_body_size {
            queue!(self.stdout, Print("\n")).expect("Can print to stdout");
        }

        // App footer
        queue!(self.stdout, Print("\n"),).expect("Can print to stdout");
        if let Some(error) = options.error {
            queue!(
                self.stdout,
                cursor::MoveToColumn(0),
                style::PrintStyledContent(format!("{}\n", error).white().on_red(),)
            )
            .expect("Can print to stdout and move cursor");
        } else {
            queue!(self.stdout, Print("\n")).expect("Can print to stdout and move cursor");
        }

        queue!(
            self.stdout,
            cursor::MoveToColumn(0),
            style::PrintStyledContent(INSTRUCTIONS.cyan()),
            cursor::MoveTo(0, HEADER_LINES - 1)
        )
        .expect("Can print to stdout and move cursor");

        self.stdout.flush().expect("Can flush stdout");
    }

    fn set_event_sender(&mut self, event_sender: mpsc::Sender<TodoRendererEvent>) {
        self.event_sender = Some(event_sender);
    }
}
