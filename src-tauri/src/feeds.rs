//! Local RSS/Atom subscriptions. Feed content is stored and rendered as text.
use quick_xml::{events::Event, Reader};
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
fn parse(xml: &str, base: &str) -> Result<Vec<Entry>, String> {
    let base = url::Url::parse(base).map_err(|_| "feeds.invalid_url")?;
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);
    let mut result = Vec::new();
    let mut current: Option<Entry> = None;
    let mut field = String::new();
    let mut format = false;
    let mut depth = 0usize;
    loop {
        match reader.read_event().map_err(|_| "feeds.invalid_xml")? {
            Event::Start(e) => {
                depth += 1;
                if depth > 64 {
                    return Err("feeds.invalid_xml".into());
                }
                let name = e.local_name().as_ref().to_ascii_lowercase();
                if matches!(name.as_str(), "rss" | "feed" | "rdf") {
                    format = true;
                }
                if matches!(name.as_str(), "item" | "entry") {
                    current = Some(Entry::default());
                }
                if current.is_some() && name == "link" {
                    for attr in e.attributes().flatten() {
                        if attr.key.as_ref() == "href" {
                            current.as_mut().unwrap().url = attr
                                .normalized_value(quick_xml::XmlVersion::Implicit1_0)
                                .map_err(|_| "feeds.invalid_xml")?
                                .into_owned();
                        }
                    }
                }
                field = name;
            }
            Event::Empty(e) => {
                if matches!(e.local_name().as_ref(), "rss" | "feed" | "RDF") {
                    format = true;
                }
                if current.is_some() && e.local_name().as_ref() == "link" {
                    for attr in e.attributes().flatten() {
                        if attr.key.as_ref() == "href" {
                            current.as_mut().unwrap().url = attr
                                .normalized_value(quick_xml::XmlVersion::Implicit1_0)
                                .map_err(|_| "feeds.invalid_xml")?
                                .into_owned();
                        }
                    }
                }
            }
            Event::Text(e) => {
                if let Some(entry) = &mut current {
                    let text =
                        quick_xml::escape::unescape(e.as_ref()).map_err(|_| "feeds.invalid_xml")?;
                    match field.as_str() {
                        "title" => entry.title.push_str(&text),
                        "link" => entry.url.push_str(&text),
                        "description" | "summary" | "content" => entry.text.push_str(&text),
                        _ => {}
                    }
                }
            }
            Event::GeneralRef(e) => {
                if let Some(entry) = &mut current {
                    let encoded = format!("&{};", e.as_ref());
                    let text =
                        quick_xml::escape::unescape(&encoded).map_err(|_| "feeds.invalid_xml")?;
                    match field.as_str() {
                        "title" => entry.title.push_str(&text),
                        "link" => entry.url.push_str(&text),
                        "description" | "summary" | "content" => entry.text.push_str(&text),
                        _ => {}
                    }
                }
            }
            Event::CData(e) => {
                if let Some(entry) = &mut current {
                    if matches!(field.as_str(), "description" | "summary" | "content") {
                        entry.text.push_str(e.as_ref());
                    }
                }
            }
            Event::End(e) => {
                depth = depth.checked_sub(1).ok_or("feeds.invalid_xml")?;
                let name = e.local_name().as_ref().to_ascii_lowercase();
                if matches!(name.as_str(), "item" | "entry") {
                    if let Some(mut entry) = current.take() {
                        if let Ok(link) = base.join(entry.url.trim()) {
                            if matches!(link.scheme(), "http" | "https")
                                && link.username().is_empty()
                                && link.password().is_none()
                                && !entry.url.trim().is_empty()
                            {
                                entry.url = link.to_string();
                                entry.title = entry.title.chars().take(500).collect();
                                entry.text = entry.text.chars().take(20_000).collect();
                                result.push(entry);
                            }
                        }
                    }
                }
                field.clear();
            }
            Event::DocType(_) => return Err("feeds.unsupported_xml".into()),
            Event::Eof => {
                if depth != 0 {
                    return Err("feeds.invalid_xml".into());
                }
                break;
            }
            _ => {}
        }
    }
    if !format {
        return Err("feeds.invalid_format".into());
    }
    result.truncate(200);
    Ok(result)
}
pub fn refresh(app: &AppHandle, id: i64) -> Result<(), String> {
    let state = app.state::<crate::state::browser_state::BrowserState>();
    let feed = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        list(db.as_ref().ok_or("Database unavailable")?)?
            .into_iter()
            .find(|v| v.id == id)
            .ok_or("feeds.not_found")?
    };
    let result = (|| {
        let client = reqwest::blocking::Client::builder()
            .connect_timeout(Duration::from_secs(10))
            .timeout(Duration::from_secs(25))
            .redirect(reqwest::redirect::Policy::limited(5))
            .build()
            .map_err(|_| "feeds.network_error")?;
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
    let db = state.db.lock().map_err(|e| e.to_string())?;
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
        let ids = app.try_state::<crate::state::browser_state::BrowserState>().and_then(|state| state.db.lock().ok().and_then(|db| db.as_ref().and_then(|c| c.prepare("SELECT id FROM rss_feeds WHERE next_update<=?1 ORDER BY next_update LIMIT 20").ok().and_then(|mut s| s.query_map([now()], |row| row.get::<_,i64>(0)).ok().map(|rows| rows.filter_map(Result::ok).collect::<Vec<_>>()))))).unwrap_or_default();
        for id in ids {
            let _ = refresh(&app, id);
        }
        std::thread::sleep(Duration::from_secs(60));
    });
}

#[cfg(test)]
mod tests {
    use super::*;
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
