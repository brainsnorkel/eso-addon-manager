use rusqlite::Connection;
use std::sync::Mutex;

/// Application state shared across Tauri commands
pub struct AppState {
    pub db: Mutex<Connection>,
}

impl AppState {
    pub fn new(db: Connection) -> Self {
        Self { db: Mutex::new(db) }
    }
}
