use crate::updates::model::UpdateChannel;
use rusqlite::Connection;

pub fn load(
    conn: &Connection,
    initial: UpdateChannel,
) -> Result<(UpdateChannel, Option<i64>), String> {
    super::migrations::apply(conn, "update-preferences-v1", |conn| {
        conn.execute_batch(
            "CREATE TABLE update_preferences (
            id INTEGER PRIMARY KEY CHECK(id = 1),
            channel TEXT NOT NULL CHECK(channel IN ('stable', 'beta')),
            last_checked INTEGER
        );",
        )
        .map_err(|e| e.to_string())
    })?;
    super::migrations::apply(conn, "update-auto-update-v1", |conn| {
        conn.execute_batch(
            "ALTER TABLE update_preferences ADD COLUMN auto_update INTEGER NOT NULL DEFAULT 1 CHECK(auto_update IN (0, 1));",
        )
        .map_err(|e| e.to_string())
    })?;
    conn.execute(
        "INSERT OR IGNORE INTO update_preferences (id, channel) VALUES (1, ?1)",
        [initial.as_str()],
    )
    .map_err(|e| e.to_string())?;
    conn.query_row(
        "SELECT channel, last_checked FROM update_preferences WHERE id = 1",
        [],
        |row| {
            let channel: String = row.get(0)?;
            Ok((
                if channel == "beta" {
                    UpdateChannel::Beta
                } else {
                    UpdateChannel::Stable
                },
                row.get(1)?,
            ))
        },
    )
    .map_err(|e| e.to_string())
}
/// Whether updates download in the background and install at the next start. On unless turned off.
/// `load` must have run first: it creates the column.
pub fn load_auto_update(conn: &Connection) -> Result<bool, String> {
    conn.query_row(
        "SELECT auto_update FROM update_preferences WHERE id = 1",
        [],
        |row| row.get::<_, i64>(0),
    )
    .map(|value| value != 0)
    .map_err(|e| e.to_string())
}
pub fn set_auto_update(conn: &Connection, enabled: bool) -> Result<(), String> {
    conn.execute(
        "UPDATE update_preferences SET auto_update = ?1 WHERE id = 1",
        [i64::from(enabled)],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}
pub fn set_channel(conn: &Connection, channel: UpdateChannel) -> Result<(), String> {
    conn.execute(
        "UPDATE update_preferences SET channel = ?1 WHERE id = 1",
        [channel.as_str()],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}
pub fn record_check(conn: &Connection, timestamp: i64) -> Result<(), String> {
    conn.execute(
        "UPDATE update_preferences SET last_checked = ?1 WHERE id = 1",
        [timestamp],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn preferences_preserve_channel_across_versions_and_restarts() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("CREATE TABLE app_migrations (name TEXT PRIMARY KEY)")
            .unwrap();
        assert_eq!(
            load(&conn, UpdateChannel::initial("1.1.0-beta.1")).unwrap(),
            (UpdateChannel::Beta, None)
        );
        set_channel(&conn, UpdateChannel::Stable).unwrap();
        record_check(&conn, 42).unwrap();
        assert_eq!(
            load(&conn, UpdateChannel::Beta).unwrap(),
            (UpdateChannel::Stable, Some(42))
        );
    }
    #[test]
    fn automatic_updates_are_on_by_default_including_old_profiles_and_persist_when_turned_off() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("CREATE TABLE app_migrations (name TEXT PRIMARY KEY)")
            .unwrap();
        // A profile from before the setting existed: the table and a row, without the column.
        conn.execute_batch(
            "CREATE TABLE update_preferences (id INTEGER PRIMARY KEY CHECK(id = 1), channel TEXT NOT NULL CHECK(channel IN ('stable', 'beta')), last_checked INTEGER);
             INSERT INTO update_preferences (id, channel) VALUES (1, 'beta');
             INSERT INTO app_migrations (name) VALUES ('update-preferences-v1');",
        )
        .unwrap();
        assert_eq!(load(&conn, UpdateChannel::Stable).unwrap().0, UpdateChannel::Beta);
        assert!(load_auto_update(&conn).unwrap(), "existing profiles get the default");
        set_auto_update(&conn, false).unwrap();
        assert_eq!(load(&conn, UpdateChannel::Stable).unwrap().0, UpdateChannel::Beta);
        assert!(!load_auto_update(&conn).unwrap(), "the choice survives later loads");
        set_auto_update(&conn, true).unwrap();
        assert!(load_auto_update(&conn).unwrap());
    }
}
