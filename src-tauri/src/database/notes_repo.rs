use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
#[derive(Clone, Serialize, Deserialize)]
pub struct PageNote {
    pub url: String,
    pub title: String,
    pub text: String,
    pub updated_at: i64,
}
/// Bounded list payload; the full text is available only through get().
#[derive(Clone, Serialize)]
pub struct NoteSummary {
    pub url: String,
    pub title: String,
    pub preview: String,
    pub updated_at: i64,
}
pub fn normalize(raw: &str) -> Result<String, String> {
    let mut url = url::Url::parse(raw).map_err(|_| "Invalid page URL")?;
    if !matches!(url.scheme(), "http" | "https") {
        return Err("Notes require an HTTP/HTTPS page".into());
    }
    url.set_fragment(None);
    Ok(url.to_string())
}
pub fn migrate(conn: &Connection) -> Result<(), String> {
    super::migrations::apply(conn, "page-notes-v1", |conn| {
        conn.execute_batch("CREATE TABLE page_notes(url TEXT PRIMARY KEY,title TEXT NOT NULL,text TEXT NOT NULL,updated_at INTEGER NOT NULL)").map_err(|e|e.to_string())
    })
}
pub fn get(conn: &Connection, url: &str) -> Result<Option<PageNote>, String> {
    conn.query_row(
        "SELECT url,title,text,updated_at FROM page_notes WHERE url=?1",
        [normalize(url)?],
        decode,
    )
    .optional()
    .map_err(|e| e.to_string())
}
fn decode(row: &rusqlite::Row<'_>) -> rusqlite::Result<PageNote> {
    Ok(PageNote {
        url: row.get(0)?,
        title: row.get(1)?,
        text: row.get(2)?,
        updated_at: row.get(3)?,
    })
}
pub fn search(conn: &Connection, query: &str) -> Result<Vec<NoteSummary>, String> {
    let query = format!(
        "%{}%",
        query
            .replace('!', "!!")
            .replace('%', "!%")
            .replace('_', "!_")
    );
    let mut stmt=conn.prepare("SELECT url,substr(title,1,256),substr(text,1,180),updated_at FROM page_notes WHERE title LIKE ?1 ESCAPE '!' OR url LIKE ?1 ESCAPE '!' OR text LIKE ?1 ESCAPE '!' ORDER BY updated_at DESC,url LIMIT 100").map_err(|e|e.to_string())?;
    let rows = stmt
        .query_map([query], |row| {
            Ok(NoteSummary {
                url: row.get(0)?,
                title: row.get(1)?,
                preview: row.get(2)?,
                updated_at: row.get(3)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    Ok(rows)
}
pub fn save(conn: &Connection, url: &str, title: &str, text: &str) -> Result<(), String> {
    if text.len() > 200_000 {
        return Err("Note exceeds 200 KB".into());
    }
    conn.execute("INSERT INTO page_notes VALUES(?1,?2,?3,unixepoch()) ON CONFLICT(url) DO UPDATE SET title=excluded.title,text=excluded.text,updated_at=excluded.updated_at",params![normalize(url)?,title,text]).map_err(|e|e.to_string())?;
    Ok(())
}
pub fn delete(conn: &Connection, url: &str) -> Result<(), String> {
    conn.execute("DELETE FROM page_notes WHERE url=?1", [normalize(url)?])
        .map_err(|e| e.to_string())?;
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn notes_share_fragments_but_keep_query_and_search_literals() {
        let conn = Connection::open_in_memory().unwrap();
        migrate(&conn).unwrap();
        save(
            &conn,
            "https://example.test/a?q=1#first",
            "Title",
            "literal 100% _ <script>",
        )
        .unwrap();
        assert!(get(&conn, "https://example.test/a?q=1#second")
            .unwrap()
            .is_some());
        assert!(get(&conn, "https://example.test/a?q=2").unwrap().is_none());
        assert_eq!(search(&conn, "100%").unwrap().len(), 1);
        assert!(search(&conn, "missing%").unwrap().is_empty());
        delete(&conn, "https://example.test/a?q=1#third").unwrap();
        assert!(search(&conn, "").unwrap().is_empty());
    }

    #[test]
    fn large_note_lists_return_unicode_previews_and_open_the_complete_document() {
        let conn = Connection::open_in_memory().unwrap();
        migrate(&conn).unwrap();
        let text = "🦀".repeat(50_000);
        let title = "é".repeat(500);
        for i in 0..100 {
            save(&conn, &format!("https://example.test/{i}"), &title, &text).unwrap();
        }
        let summaries = search(&conn, "").unwrap();
        assert_eq!(summaries.len(), 100);
        assert!(summaries
            .iter()
            .all(|n| n.preview.chars().count() == 180 && n.title.chars().count() == 256));
        let payload = serde_json::to_vec(&summaries).unwrap();
        assert!(
            payload.len() < 150_000,
            "bounded summaries: {} bytes",
            payload.len()
        );
        assert_eq!(get(&conn, &summaries[0].url).unwrap().unwrap().text, text);
    }
}
