use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HistoryRetention {
    OneMonth,
    ThreeMonths,
    SixMonths,
    TwelveMonths,
    Always,
    Never,
}

impl HistoryRetention {
    pub fn from_preference(value: Option<String>) -> Self {
        match value.as_deref() {
            Some("one_month") => Self::OneMonth,
            Some("six_months") => Self::SixMonths,
            Some("twelve_months") => Self::TwelveMonths,
            Some("always") => Self::Always,
            Some("never") => Self::Never,
            _ => Self::ThreeMonths,
        }
    }

    pub fn months(self) -> Option<u32> {
        match self {
            Self::OneMonth => Some(1),
            Self::ThreeMonths => Some(3),
            Self::SixMonths => Some(6),
            Self::TwelveMonths => Some(12),
            Self::Always | Self::Never => None,
        }
    }

    pub fn from_months(months: u64) -> Self {
        match months {
            1 => Self::OneMonth,
            6 => Self::SixMonths,
            12 => Self::TwelveMonths,
            0 => Self::Always,
            u64::MAX => Self::Never,
            _ => Self::ThreeMonths,
        }
    }

    pub fn state_value(self) -> u64 {
        if self == Self::Never {
            u64::MAX
        } else {
            u64::from(self.months().unwrap_or(0))
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub id: i64,
    pub title: String,
    pub url: String,
    pub visited_at: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct HistoryCursor {
    pub visited_at: i64,
    pub id: i64,
}

pub fn current_unix_timestamp() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

pub fn record_visit(
    conn: &Connection,
    title: &str,
    url: &str,
    retention: HistoryRetention,
) -> Result<(), String> {
    if retention == HistoryRetention::Never || url.trim().is_empty() || url.starts_with("about:") {
        return Ok(());
    }

    let now = current_unix_timestamp();
    prune_expired(conn, retention, now)?;
    conn.execute(
        "INSERT INTO history (title, url, visited_at) VALUES (?1, ?2, ?3)",
        params![title, url, now],
    )
    .map_err(|e| format!("Failed to record history visit: {}", e))?;

    Ok(())
}

pub fn prune_expired(
    conn: &Connection,
    retention: HistoryRetention,
    now: i64,
) -> Result<(), String> {
    let Some(months) = retention.months() else {
        return Ok(());
    };
    let cutoff = now.saturating_sub(i64::from(months) * 30 * 24 * 60 * 60);
    conn.execute("DELETE FROM history WHERE visited_at < ?1", [cutoff])
        .map_err(|e| format!("Failed to prune browsing history: {e}"))?;
    Ok(())
}

pub fn get_recent_history(conn: &Connection, limit: usize) -> Result<Vec<HistoryEntry>, String> {
    search_history(conn, limit, "", None)
}

pub fn search_history(
    conn: &Connection,
    limit: usize,
    query: &str,
    before: Option<&HistoryCursor>,
) -> Result<Vec<HistoryEntry>, String> {
    let pattern = format!(
        "%{}%",
        query
            .trim()
            .replace('!', "!!")
            .replace('%', "!%")
            .replace('_', "!_")
    );
    let mut stmt = conn
        .prepare_cached(
            "SELECT id, title, url, visited_at
             FROM history
             WHERE (?2 = '%%' OR title LIKE ?2 ESCAPE '!' OR url LIKE ?2 ESCAPE '!')
               AND (?3 IS NULL OR (visited_at, id) < (?3, ?4))
             ORDER BY visited_at DESC, id DESC
             LIMIT ?1",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map(
            params![
                limit.clamp(1, 200) as i64,
                pattern,
                before.map(|cursor| cursor.visited_at),
                before.map(|cursor| cursor.id)
            ],
            |row| {
                Ok(HistoryEntry {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    url: row.get(2)?,
                    visited_at: row.get(3)?,
                })
            },
        )
        .map_err(|e| e.to_string())?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}

pub fn clear_all_history(conn: &Connection) -> Result<(), String> {
    conn.execute("DELETE FROM history", [])
        .map_err(|e| format!("Failed to clear history: {}", e))?;
    Ok(())
}

/// The Unix time `days` days ago: what is older than that is "older than `days` days".
pub fn days_ago(days: u32) -> i64 {
    current_unix_timestamp() - i64::from(days) * 86_400
}

/// Bytes the history takes that a cleaning would free: the visits, the recently closed tabs and, when the
/// whole history goes (no `cutoff`), the saved site icons. A row counts its text plus a small fixed overhead.
pub fn estimate_history_bytes(conn: &Connection, cutoff: Option<i64>) -> Result<u64, String> {
    let sum = |sql: &str| -> Result<i64, String> {
        conn.query_row(sql, [cutoff.unwrap_or(i64::MAX)], |row| {
            row.get::<_, i64>(0)
        })
        .map_err(|e| e.to_string())
    };
    let mut total = sum("SELECT COALESCE(SUM(LENGTH(title) + LENGTH(url) + 24), 0) FROM history WHERE visited_at < ?1")?
        + sum("SELECT COALESCE(SUM(LENGTH(url) + LENGTH(title) + LENGTH(workspace_name) + 32), 0) FROM closed_tabs WHERE closed_at < ?1")?;
    if cutoff.is_none() {
        total += sum(
            "SELECT COALESCE(SUM(LENGTH(site_key) + LENGTH(png)), 0) FROM site_icons WHERE ?1 = ?1",
        )?;
    }
    Ok(total.max(0) as u64)
}

/// Clears only what is older than `cutoff` (a Unix time): the visits and the recently closed tabs.
/// Site icons stay, since newer visits may still use them. Caller owns the transaction.
pub fn clear_history_older_than(conn: &Connection, cutoff: i64) -> Result<(), String> {
    conn.execute("DELETE FROM history WHERE visited_at < ?1", [cutoff])
        .map_err(|e| format!("Failed to clear old history: {}", e))?;
    crate::database::closed_tabs_repo::clear_before(conn, cutoff)
}

/// Caller owns the transaction so all local browsing history clears together.
pub fn clear_history_data(conn: &Connection) -> Result<(), String> {
    clear_all_history(conn)?;
    crate::database::site_icons_repo::clear(conn)?;
    crate::database::closed_tabs_repo::clear(conn)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn the_estimate_counts_only_what_a_cleaning_would_remove() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("CREATE TABLE history(id INTEGER PRIMARY KEY,title TEXT NOT NULL,url TEXT NOT NULL,visited_at INTEGER NOT NULL); CREATE TABLE closed_tabs(id INTEGER PRIMARY KEY, url TEXT NOT NULL, title TEXT NOT NULL, workspace_name TEXT NOT NULL, closed_at INTEGER NOT NULL); CREATE TABLE site_icons(site_key TEXT PRIMARY KEY, png BLOB NOT NULL, updated_at INTEGER NOT NULL);").unwrap();
        let now = current_unix_timestamp();
        conn.execute(
            "INSERT INTO history(title,url,visited_at) VALUES('ab','cde',?1),('ab','cde',?2)",
            params![now - 86_400, now - 30 * 86_400],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO site_icons(site_key,png,updated_at) VALUES('k', x'00010203', 0)",
            [],
        )
        .unwrap();
        let row = (2 + 3 + 24) as u64;
        assert_eq!(
            estimate_history_bytes(&conn, Some(days_ago(5))).unwrap(),
            row,
            "only the old visit, and no icons"
        );
        assert_eq!(
            estimate_history_bytes(&conn, None).unwrap(),
            2 * row + 1 + 4,
            "everything, with the saved icons"
        );
        assert_eq!(
            estimate_history_bytes(&conn, Some(days_ago(90))).unwrap(),
            0
        );
    }

    #[test]
    fn clearing_by_age_keeps_the_newer_visits() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("CREATE TABLE history(id INTEGER PRIMARY KEY,title TEXT NOT NULL,url TEXT NOT NULL,visited_at INTEGER NOT NULL); CREATE TABLE closed_tabs(id INTEGER PRIMARY KEY, closed_at INTEGER NOT NULL);").unwrap();
        let now = current_unix_timestamp();
        for (title, age_days) in [("fresh", 1), ("middle", 7), ("old", 30)] {
            conn.execute(
                "INSERT INTO history(title,url,visited_at) VALUES(?1,'https://example.test/',?2)",
                params![title, now - age_days * 86_400],
            )
            .unwrap();
        }
        conn.execute(
            "INSERT INTO closed_tabs(closed_at) VALUES(?1),(?2)",
            params![now - 86_400, now - 30 * 86_400],
        )
        .unwrap();
        clear_history_older_than(&conn, days_ago(5)).unwrap();
        let left: Vec<String> = conn
            .prepare("SELECT title FROM history ORDER BY visited_at DESC")
            .unwrap()
            .query_map([], |row| row.get(0))
            .unwrap()
            .map(Result::unwrap)
            .collect();
        assert_eq!(left, ["fresh"]);
        assert_eq!(
            conn.query_row("SELECT COUNT(*) FROM closed_tabs", [], |row| row
                .get::<_, i64>(0))
                .unwrap(),
            1
        );
    }

    #[test]
    fn never_preserves_existing_visits_and_does_not_record_new_ones() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("CREATE TABLE history(id INTEGER PRIMARY KEY,title TEXT NOT NULL,url TEXT NOT NULL,visited_at INTEGER NOT NULL); INSERT INTO history(title,url,visited_at) VALUES('old','https://example.test/',1);").unwrap();
        prune_expired(&conn, HistoryRetention::Never, current_unix_timestamp()).unwrap();
        record_visit(
            &conn,
            "new",
            "https://new.example/",
            HistoryRetention::Never,
        )
        .unwrap();
        assert_eq!(get_recent_history(&conn, 10).unwrap().len(), 1);
        assert_eq!(
            HistoryRetention::from_months(HistoryRetention::Never.state_value()),
            HistoryRetention::Never
        );
        assert_eq!(
            HistoryRetention::from_preference(Some("never".into())),
            HistoryRetention::Never
        );
    }
    #[test]
    fn searches_older_visits_and_paginates_equal_timestamps_without_duplicates() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("CREATE TABLE history (id INTEGER PRIMARY KEY, title TEXT, url TEXT, visited_at INTEGER); CREATE INDEX idx_history_recent ON history(visited_at DESC, id DESC);").unwrap();
        for id in 1..=300 {
            conn.execute(
                "INSERT INTO history VALUES (?1, ?2, 'https://example.test', 100)",
                params![
                    id,
                    if id == 1 {
                        "Older 100%_match"
                    } else {
                        "Recent"
                    }
                ],
            )
            .unwrap();
        }
        let first = search_history(&conn, 60, "", None).unwrap();
        let cursor = HistoryCursor {
            visited_at: first[59].visited_at,
            id: first[59].id,
        };
        let next = search_history(&conn, 60, "", Some(&cursor)).unwrap();
        assert_eq!(next[0].id, 240);
        assert_eq!(
            search_history(&conn, 60, "100%_match", None).unwrap()[0].id,
            1
        );
        assert_eq!(
            search_history(&conn, usize::MAX, "", None).unwrap().len(),
            200
        );
        let plan: String = conn.query_row("EXPLAIN QUERY PLAN SELECT * FROM history ORDER BY visited_at DESC, id DESC LIMIT 60", [], |row| row.get(3)).unwrap();
        assert!(plan.contains("idx_history_recent"));
        assert!(!plan.contains("TEMP B-TREE"));
    }

    #[test]
    fn retention_prunes_only_history_older_than_the_selected_period() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("CREATE TABLE history (id INTEGER PRIMARY KEY, title TEXT, url TEXT, visited_at INTEGER);").unwrap();
        conn.execute(
            "INSERT INTO history VALUES (1, 'old', 'https://old.test', 1)",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO history VALUES (2, 'recent', 'https://recent.test', 100)",
            [],
        )
        .unwrap();
        let now = 30 * 24 * 60 * 60 + 100;
        prune_expired(&conn, HistoryRetention::OneMonth, now).unwrap();
        let remaining: i64 = conn
            .query_row("SELECT count(*) FROM history", [], |row| row.get(0))
            .unwrap();
        assert_eq!(remaining, 1);
        prune_expired(&conn, HistoryRetention::Always, now).unwrap();
        let retained: i64 = conn
            .query_row("SELECT count(*) FROM history", [], |row| row.get(0))
            .unwrap();
        assert_eq!(retained, 1);
    }

    #[test]
    fn clearing_history_does_not_touch_other_profile_data() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE history (id INTEGER PRIMARY KEY, title TEXT, url TEXT, visited_at INTEGER);
             CREATE TABLE site_data (id INTEGER PRIMARY KEY, value TEXT);
             INSERT INTO history VALUES (1, 'visited', 'https://example.test', 100);
             INSERT INTO site_data VALUES (1, 'keep');",
        )
        .unwrap();

        clear_all_history(&conn).unwrap();

        let history_count: i64 = conn
            .query_row("SELECT count(*) FROM history", [], |row| row.get(0))
            .unwrap();
        let site_data: String = conn
            .query_row("SELECT value FROM site_data WHERE id = 1", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(history_count, 0);
        assert_eq!(site_data, "keep");
    }

    #[test]
    fn clearing_history_data_rolls_back_together_on_failure() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("CREATE TABLE history(id INTEGER PRIMARY KEY,title TEXT,url TEXT,visited_at INTEGER); CREATE TABLE site_icons(site_key TEXT PRIMARY KEY); CREATE TABLE closed_tabs(id INTEGER PRIMARY KEY); INSERT INTO history VALUES(1,'Page','https://example.test/',1); INSERT INTO site_icons VALUES('example.test'); INSERT INTO closed_tabs VALUES(1); CREATE TRIGGER fail_icon_delete BEFORE DELETE ON site_icons BEGIN SELECT RAISE(ABORT,'fixture failure'); END;").unwrap();
        {
            let tx = conn.unchecked_transaction().unwrap();
            assert!(clear_history_data(&tx).is_err());
        }
        assert_eq!(
            conn.query_row("SELECT COUNT(*) FROM history", [], |row| row
                .get::<_, i64>(0))
                .unwrap(),
            1
        );
        assert_eq!(
            conn.query_row("SELECT COUNT(*) FROM site_icons", [], |row| row
                .get::<_, i64>(0))
                .unwrap(),
            1
        );
        conn.execute_batch("DROP TRIGGER fail_icon_delete").unwrap();
        let tx = conn.unchecked_transaction().unwrap();
        clear_history_data(&tx).unwrap();
        tx.commit().unwrap();
        assert_eq!(
            conn.query_row("SELECT COUNT(*) FROM history", [], |row| row
                .get::<_, i64>(0))
                .unwrap(),
            0
        );
        assert_eq!(
            conn.query_row("SELECT COUNT(*) FROM site_icons", [], |row| row
                .get::<_, i64>(0))
                .unwrap(),
            0
        );
        assert_eq!(
            conn.query_row("SELECT COUNT(*) FROM closed_tabs", [], |row| row
                .get::<_, i64>(0))
                .unwrap(),
            0
        );
    }
}
