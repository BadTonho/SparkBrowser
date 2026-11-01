use rusqlite::{params, Connection};

pub const USAGE_WINDOW_SECS: u64 = 7 * 24 * 60 * 60;
pub const FREQUENT_ACTIVATION_THRESHOLD: u64 = 10;

pub fn reset_tab_history(conn: &Connection, tab_id: &str) -> Result<(), String> {
    conn.execute("DELETE FROM tab_usage_events WHERE tab_id = ?1", [tab_id])
        .map_err(|e| format!("Failed to reset tab activation history: {e}"))?;
    Ok(())
}

pub fn record_activation(conn: &Connection, tab_id: &str, activated_at: u64) -> Result<(), String> {
    let cutoff = activated_at.saturating_sub(USAGE_WINDOW_SECS);
    conn.execute(
        "INSERT INTO tab_usage_events (tab_id, activated_at) VALUES (?1, ?2)",
        params![tab_id, activated_at as i64],
    )
    .map_err(|e| format!("Failed to record tab activation: {e}"))?;
    conn.execute(
        "DELETE FROM tab_usage_events WHERE activated_at < ?1",
        [cutoff as i64],
    )
    .map_err(|e| format!("Failed to prune expired tab activations: {e}"))?;
    Ok(())
}

pub fn is_frequently_used(conn: &Connection, tab_id: &str, now: u64) -> Result<bool, String> {
    let cutoff = now.saturating_sub(USAGE_WINDOW_SECS);
    let count: u64 = conn
        .query_row(
            "SELECT COUNT(*) FROM tab_usage_events
             WHERE tab_id = ?1 AND activated_at >= ?2",
            params![tab_id, cutoff as i64],
            |row| row.get(0),
        )
        .map_err(|e| format!("Failed to read tab activation history: {e}"))?;
    Ok(count >= FREQUENT_ACTIVATION_THRESHOLD)
}

pub fn prune_expired(conn: &Connection, now: u64) -> Result<(), String> {
    let cutoff = now.saturating_sub(USAGE_WINDOW_SECS);
    conn.execute(
        "DELETE FROM tab_usage_events WHERE activated_at < ?1",
        [cutoff as i64],
    )
    .map_err(|e| format!("Failed to prune expired tab activations: {e}"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE tab_usage_events (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                tab_id TEXT NOT NULL,
                activated_at INTEGER NOT NULL
            );",
        )
        .unwrap();
        conn
    }

    #[test]
    fn frequent_tabs_need_ten_activations_within_the_rolling_week() {
        let conn = setup_db();
        let now = 2_000_000;

        for _ in 0..9 {
            record_activation(&conn, "tab-1", now).unwrap();
        }
        assert!(!is_frequently_used(&conn, "tab-1", now).unwrap());

        record_activation(&conn, "tab-1", now).unwrap();
        assert!(is_frequently_used(&conn, "tab-1", now).unwrap());
        assert!(!is_frequently_used(&conn, "tab-2", now).unwrap());
    }

    #[test]
    fn expired_activations_are_pruned_and_new_tab_ids_start_fresh() {
        let conn = setup_db();
        let now = 2_000_000;
        for _ in 0..FREQUENT_ACTIVATION_THRESHOLD {
            record_activation(&conn, "tab-1", now - USAGE_WINDOW_SECS - 1).unwrap();
        }
        assert!(!is_frequently_used(&conn, "tab-1", now).unwrap());
        prune_expired(&conn, now).unwrap();
        let remaining: u64 = conn
            .query_row("SELECT COUNT(*) FROM tab_usage_events", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(remaining, 0);

        reset_tab_history(&conn, "tab-1").unwrap();
        record_activation(&conn, "tab-1", now).unwrap();
        assert!(!is_frequently_used(&conn, "tab-1", now).unwrap());
    }
}
