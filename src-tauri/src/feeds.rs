//! Local RSS/Atom subscriptions. Feed content is stored and rendered as text.
mod parser;
use parser::parse;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    io::Read,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tauri::{AppHandle, Manager};

#[derive(Clone, Serialize, Deserialize)]
pub struct Feed {
    pub id: i64,
    pub title: String,
    pub url: String,
    pub interval_minutes: u32,
    pub last_error: Option<String>,
}
#[derive(Clone, Default, Serialize, Deserialize)]
pub struct Entry {
    pub title: String,
    pub url: String,
    pub text: String,
}
fn now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}
pub fn migrate(conn: &Connection) -> Result<(), String> {
    crate::database::migrations::apply(conn, "rss-feeds-v1", |c| {
        c.execute_batch("CREATE TABLE IF NOT EXISTS rss_feeds(id INTEGER PRIMARY KEY, title TEXT NOT NULL, url TEXT NOT NULL UNIQUE, interval_minutes INTEGER NOT NULL DEFAULT 60, next_update INTEGER NOT NULL DEFAULT 0, last_error TEXT); CREATE TABLE IF NOT EXISTS rss_entries(feed_id INTEGER NOT NULL, entry_key TEXT NOT NULL, title TEXT NOT NULL, url TEXT NOT NULL, body TEXT NOT NULL, seen_at INTEGER NOT NULL, PRIMARY KEY(feed_id,entry_key), FOREIGN KEY(feed_id) REFERENCES rss_feeds(id) ON DELETE CASCADE); CREATE INDEX IF NOT EXISTS rss_entries_feed_time ON rss_entries(feed_id,seen_at DESC);").map_err(|e| e.to_string())
    })
}
pub fn valid_url(raw: &str) -> Result<String, String> {
    let u = url::Url::parse(raw.trim()).map_err(|_| "feeds.invalid_url")?;
    if !matches!(u.scheme(), "http" | "https")
        || u.host_str().is_none()
        || !u.username().is_empty()
        || u.password().is_some()
    {
        return Err("feeds.invalid_url".into());
    }
    Ok(u.to_string())
}
pub fn list(conn: &Connection) -> Result<Vec<Feed>, String> {
    let mut stmt = conn
        .prepare("SELECT id,title,url,interval_minutes,last_error FROM rss_feeds ORDER BY id")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            Ok(Feed {
                id: row.get(0)?,
                title: row.get(1)?,
                url: row.get(2)?,
                interval_minutes: row.get(3)?,
                last_error: row.get(4)?,
            })
        })
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<_, _>>().map_err(|e| e.to_string())
}
pub fn save(
    conn: &Connection,
    id: Option<i64>,
    title: &str,
    raw: &str,
    interval: u32,
) -> Result<(), String> {
    let url = valid_url(raw)?;
    let title = title.trim();
    if title.is_empty() || title.chars().count() > 120 || !(15..=1440).contains(&interval) {
        return Err("feeds.invalid_settings".into());
    }
    match id {
        Some(id) => {
            let tx = conn.unchecked_transaction().map_err(|e| e.to_string())?;
            tx.execute("DELETE FROM rss_entries WHERE feed_id=?1 AND EXISTS (SELECT 1 FROM rss_feeds WHERE id=?1 AND url<>?2)", params![id,url]).map_err(|e| e.to_string())?;
            let count = tx.execute("UPDATE rss_feeds SET title=?1,url=?2,interval_minutes=?3,next_update=0,last_error=NULL WHERE id=?4", params![title,url,interval,id]).map_err(|e| e.to_string())?;
            if count != 1 {
                return Err("feeds.not_found".into());
            }
            tx.commit().map_err(|e| e.to_string())?;
        }
        None => {
            conn.execute(
                "INSERT INTO rss_feeds(title,url,interval_minutes) VALUES (?1,?2,?3)",
                params![title, url, interval],
            )
            .map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}
pub fn remove(conn: &Connection, id: i64) -> Result<(), String> {
    let tx = conn.unchecked_transaction().map_err(|e| e.to_string())?;
    tx.execute("DELETE FROM rss_entries WHERE feed_id=?1", [id])
        .map_err(|e| e.to_string())?;
    tx.execute("DELETE FROM rss_feeds WHERE id=?1", [id])
        .map_err(|e| e.to_string())?;
    tx.commit().map_err(|e| e.to_string())
}
pub fn entries(conn: &Connection, id: i64) -> Result<Vec<Entry>, String> {
    let mut stmt = conn.prepare("SELECT title,url,body FROM rss_entries WHERE feed_id=?1 ORDER BY seen_at DESC,rowid DESC LIMIT 200").map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([id], |row| {
            Ok(Entry {
                title: row.get(0)?,
                url: row.get(1)?,
                text: row.get(2)?,
            })
        })
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<_, _>>().map_err(|e| e.to_string())
}

const REFRESH_CONCURRENCY: usize = 4;
static REFRESHING: std::sync::Mutex<std::collections::BTreeSet<i64>> =
    std::sync::Mutex::new(std::collections::BTreeSet::new());
struct RefreshPermit(i64);
impl RefreshPermit {
    fn acquire(id: i64) -> Result<Self, String> {
        let mut active = REFRESHING.lock().map_err(|_| "feeds.busy")?;
        if active.len() >= REFRESH_CONCURRENCY || !active.insert(id) {
            return Err("feeds.busy".into());
        }
        Ok(Self(id))
    }
}
impl Drop for RefreshPermit {
    fn drop(&mut self) {
        if let Ok(mut active) = REFRESHING.lock() {
            active.remove(&self.0);
        }
    }
}
fn shared_client() -> Result<&'static reqwest::blocking::Client, String> {
    static CLIENT: std::sync::OnceLock<Result<reqwest::blocking::Client, String>> =
        std::sync::OnceLock::new();
    CLIENT
        .get_or_init(|| {
            reqwest::blocking::Client::builder()
                .connect_timeout(Duration::from_secs(10))
                .timeout(Duration::from_secs(25))
                .redirect(reqwest::redirect::Policy::limited(5))
                .build()
                .map_err(|_| "feeds.network_error".into())
        })
        .as_ref()
        .map_err(Clone::clone)
}
fn refresh_batch(ids: &[i64], refresh: impl Fn(i64) + Sync) {
    // Four long-lived workers per batch; a slow feed never serializes the rest.
    let next = std::sync::atomic::AtomicUsize::new(0);
    std::thread::scope(|scope| {
        for _ in 0..REFRESH_CONCURRENCY.min(ids.len()) {
            let refresh = &refresh;
            let next = &next;
            scope.spawn(move || {
                while let Some(id) =
                    ids.get(next.fetch_add(1, std::sync::atomic::Ordering::Relaxed))
                {
                    refresh(*id);
                }
            });
        }
    });
}

pub fn refresh(app: &AppHandle, id: i64) -> Result<(), String> {
    let _permit = RefreshPermit::acquire(id)?;
    let state = app.state::<crate::state::browser_state::BrowserState>();
    let feed = {
        let db = state
            .infrastructure()
            .db()
            .lock()
            .map_err(|e| e.to_string())?;
        list(db.as_ref().ok_or("Database unavailable")?)?
            .into_iter()
            .find(|v| v.id == id)
            .ok_or("feeds.not_found")?
    };
    let result = (|| {
        let client = shared_client()?;
        let response = client
            .get(&feed.url)
            .send()
            .and_then(|r| r.error_for_status())
            .map_err(|_| "feeds.network_error")?;
        if !matches!(response.url().scheme(), "http" | "https") {
            return Err("feeds.invalid_url".into());
        }
        let base = response.url().to_string();
        let mut body = Vec::new();
        response
            .take(2 * 1024 * 1024 + 1)
            .read_to_end(&mut body)
            .map_err(|_| "feeds.network_error")?;
        if body.len() > 2 * 1024 * 1024 {
            return Err("feeds.too_large".into());
        }
        parse(
            std::str::from_utf8(&body).map_err(|_| "feeds.invalid_encoding")?,
            &base,
        )
    })();
    let db = state
        .infrastructure()
        .db()
        .lock()
        .map_err(|e| e.to_string())?;
    let conn = db.as_ref().ok_or("Database unavailable")?;
    let tx = conn.unchecked_transaction().map_err(|e| e.to_string())?;
    // Ignore responses from a deleted or edited subscription.
    let current = list(&tx)?
        .into_iter()
        .find(|v| v.id == id && v.url == feed.url);
    if current.is_none() {
        return Err("feeds.changed".into());
    }
    match &result {
        Ok(items) => {
            for item in items {
                let key = format!("{:x}", Sha256::digest(item.url.as_bytes()));
                tx.execute("INSERT INTO rss_entries(feed_id,entry_key,title,url,body,seen_at) VALUES (?1,?2,?3,?4,?5,?6) ON CONFLICT(feed_id,entry_key) DO UPDATE SET title=excluded.title,body=excluded.body", params![id,key,item.title,item.url,item.text,now()]).map_err(|e| e.to_string())?;
            }
            tx.execute("DELETE FROM rss_entries WHERE feed_id=?1 AND rowid NOT IN (SELECT rowid FROM rss_entries WHERE feed_id=?1 ORDER BY seen_at DESC,rowid DESC LIMIT 200)",[id]).map_err(|e| e.to_string())?;
            tx.execute(
                "UPDATE rss_feeds SET next_update=?1,last_error=NULL WHERE id=?2",
                params![now() + i64::from(feed.interval_minutes) * 60, id],
            )
            .map_err(|e| e.to_string())?;
        }
        Err(error) => {
            tx.execute(
                "UPDATE rss_feeds SET next_update=?1,last_error=?2 WHERE id=?3",
                params![now() + i64::from(feed.interval_minutes) * 60, error, id],
            )
            .map_err(|e| e.to_string())?;
        }
    }
    tx.commit().map_err(|e| e.to_string())?;
    result.map(|_| ())
}
pub fn start(app: AppHandle) {
    std::thread::spawn(move || loop {
        let ids = app.try_state::<crate::state::browser_state::BrowserState>().and_then(|state| state.infrastructure().db().lock().ok().and_then(|db| db.as_ref().and_then(|c| c.prepare("SELECT id FROM rss_feeds WHERE next_update<=?1 ORDER BY next_update LIMIT 20").ok().and_then(|mut s| s.query_map([now()], |row| row.get::<_,i64>(0)).ok().map(|rows| rows.filter_map(Result::ok).collect::<Vec<_>>()))))).unwrap_or_default();
        refresh_batch(&ids, |id| {
            crate::error_log::ignored(refresh(&app, id));
        });
        std::thread::sleep(Duration::from_secs(60));
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn refresh_workers_run_concurrently_with_a_fixed_limit_and_reuse_connections() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        let running = AtomicUsize::new(0);
        let peak = AtomicUsize::new(0);
        let total = AtomicUsize::new(0);
        let first_wave = std::sync::Barrier::new(REFRESH_CONCURRENCY);
        refresh_batch(&(0..12).collect::<Vec<_>>(), |id| {
            let current = running.fetch_add(1, Ordering::SeqCst) + 1;
            peak.fetch_max(current, Ordering::SeqCst);
            if id < REFRESH_CONCURRENCY as i64 {
                first_wave.wait();
            }
            total.fetch_add(1, Ordering::SeqCst);
            running.fetch_sub(1, Ordering::SeqCst);
        });
        assert_eq!(total.load(Ordering::SeqCst), 12);
        assert_eq!(peak.load(Ordering::SeqCst), REFRESH_CONCURRENCY);
        assert!(std::ptr::eq(
            shared_client().unwrap(),
            shared_client().unwrap()
        ));
        let permit = RefreshPermit::acquire(-100).unwrap();
        assert!(RefreshPermit::acquire(-100).is_err());
        drop(permit);
        assert!(RefreshPermit::acquire(-100).is_ok());
    }
    #[test]
    fn parses_rss_atom_and_rejects_unsafe_xml() {
        let rss = parse("<rss><channel><item><title>News</title><link>/news</link><description>&lt;script&gt;bad&lt;/script&gt;</description></item></channel></rss>", "https://example.test/feed").unwrap();
        assert_eq!(rss[0].url, "https://example.test/news");
        assert_eq!(rss[0].text, "<script>bad</script>");
        assert_eq!(
            parse(
                "<feed><entry><title>Atom</title><link href='/atom'/></entry></feed>",
                "https://example.test/"
            )
            .unwrap()[0]
                .url,
            "https://example.test/atom"
        );
        assert!(parse(
            "<!DOCTYPE rss [<!ENTITY x SYSTEM 'file:///etc/passwd'>]><rss/>",
            "https://example.test/"
        )
        .is_err());
        assert!(parse("<rss><item>", "https://example.test/").is_err());
        assert!(parse(
            "<html><body>not a feed</body></html>",
            "https://example.test/"
        )
        .is_err());
        assert!(parse(
            "<rss><item><link>javascript:alert(1)</link></item></rss>",
            "https://example.test/"
        )
        .unwrap()
        .is_empty());
        let mut many = "<rss>".to_string();
        for id in 0..250 {
            many.push_str(&format!("<item><link>/item/{id}</link></item>"));
        }
        many.push_str("</rss>");
        assert_eq!(parse(&many, "https://example.test/").unwrap().len(), 200);
    }
    #[test]
    fn validates_and_persists_feeds_and_removes_old_entries_on_source_change() {
        let conn = Connection::open_in_memory().unwrap();
        migrate(&conn).unwrap();
        assert!(save(&conn, None, "news", "file:///tmp/feed", 60).is_err());
        assert!(save(&conn, None, "news", "https://example.test/feed", 14).is_err());
        save(&conn, None, "news", "https://example.test/feed", 60).unwrap();
        let id = list(&conn).unwrap()[0].id;
        conn.execute(
            "INSERT INTO rss_entries VALUES (?1,'key','title','https://example.test/','body',1)",
            [id],
        )
        .unwrap();
        save(&conn, Some(id), "renamed", "https://example.test/feed", 120).unwrap();
        assert_eq!(entries(&conn, id).unwrap().len(), 1);
        save(
            &conn,
            Some(id),
            "renamed",
            "https://example.test/other",
            120,
        )
        .unwrap();
        assert!(entries(&conn, id).unwrap().is_empty());
        remove(&conn, id).unwrap();
        assert!(list(&conn).unwrap().is_empty());
    }
}
