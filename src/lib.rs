mod build;
pub mod controller;
pub mod render;
pub mod store;

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
