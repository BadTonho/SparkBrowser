//! Bounded blocking work using the existing connection and transaction ordering.
use crate::state::browser_state::BrowserState;
use rusqlite::Connection;
use std::sync::OnceLock;
use tauri::{AppHandle, Manager};
use tokio::sync::Semaphore;

static DATABASE_WORK: OnceLock<std::sync::Arc<Semaphore>> = OnceLock::new();

pub async fn run<T: Send + 'static>(
    app: AppHandle,
    operation: impl FnOnce(&Connection) -> Result<T, String> + Send + 'static,
) -> Result<T, String> {
    let permit = DATABASE_WORK
        .get_or_init(|| std::sync::Arc::new(Semaphore::new(2)))
        .clone()
        .acquire_owned()
        .await
        .map_err(|e| e.to_string())?;
    tauri::async_runtime::spawn_blocking(move || {
        let _permit = permit;
        let state = app.state::<BrowserState>();
        let mut wait = state
            .infrastructure()
            .diagnostics()
            .span("database.lock_wait");
        let db = state
            .infrastructure()
            .db()
            .lock()
            .map_err(|e| e.to_string())?;
        wait.finish(true, 0);
        drop(wait);
        let conn = db.as_ref().ok_or("Database is not ready")?;
        let mut span = state
            .infrastructure()
            .diagnostics()
            .span("database.execute");
        let result = operation(conn);
        span.finish(result.is_ok(), 0);
        result
    })
    .await
    .map_err(|e| e.to_string())?
}
