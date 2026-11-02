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
}
