use crate::database::{executor, preferences_repo};
use serde::Serialize;
use tauri::AppHandle;

const KEY: &str = "experiment_compact_feed_cards_v1";
#[derive(Serialize)]
pub struct Flags {
    pub compact_feed_cards: bool,
}
#[tauri::command]
pub async fn get_experimental_flags(app: AppHandle) -> Result<Flags, String> {
    executor::run(app, |db| {
        Ok(Flags {
            compact_feed_cards: preferences_repo::load_bool(db, KEY)?,
        })
    })
    .await
}
#[tauri::command]
pub async fn set_experimental_flag(
    app: AppHandle,
    name: String,
    enabled: bool,
) -> Result<(), String> {
    if name != "compact_feed_cards" {
        return Err("experiments.unknown_flag".into());
    }
    executor::run(app, move |db| {
        preferences_repo::store(db, KEY, if enabled { "true" } else { "false" })
    })
    .await
}
#[tauri::command]
pub async fn reset_experimental_flags(app: AppHandle) -> Result<(), String> {
    executor::run(app, |db| preferences_repo::store(db, KEY, "false")).await
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn flag_defaults_off_and_can_be_reset() {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        preferences_repo::initialize_preferences_table(&conn).unwrap();
        assert!(!preferences_repo::load_bool(&conn, KEY).unwrap());
        preferences_repo::store(&conn, KEY, "true").unwrap();
        assert!(preferences_repo::load_bool(&conn, KEY).unwrap());
        preferences_repo::store(&conn, KEY, "false").unwrap();
        assert!(!preferences_repo::load_bool(&conn, KEY).unwrap());
    }
}
