use crate::database::{
    history_repo::{search_history, HistoryCursor},
    site_icons_repo,
};
use rusqlite::Connection;
use serde::Serialize;
use std::collections::HashSet;

#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct RecentSite {
    pub site_key: String,
    pub domain: String,
    pub title: String,
    pub url: String,
    pub icon: Option<String>,
}

pub fn from_url(raw: &str, title: &str) -> Option<RecentSite> {
    let key = site_icons_repo::site_key(raw)?;
    if search(raw).is_some() {
        return None;
    }
    Some(RecentSite {
        site_key: key.clone(),
        domain: key.clone(),
        title: if title.trim().is_empty() {
            key
        } else {
            title.to_string()
        },
        url: raw.into(),
        icon: None,
    })
}
fn search(raw: &str) -> Option<()> {
    let url = url::Url::parse(raw).ok()?;
    if !matches!(url.scheme(), "http" | "https")
        || !url.username().is_empty()
        || url.password().is_some()
    {
        return None;
    }
    match (url.host_str()?, url.path()) {
        ("google.com" | "www.google.com" | "google.com.br" | "www.google.com.br", "/search") => {}
        (
            "duckduckgo.com" | "www.duckduckgo.com",
            "/" | "/html/" | "/html" | "/lite/" | "/lite",
        )
        | ("html.duckduckgo.com", "/html/" | "/html")
        | ("lite.duckduckgo.com", "/lite/" | "/lite") => {}
        _ => return None,
    };
    let query = url
        .query_pairs()
        .find(|(key, _)| key == "q")?
        .1
        .trim()
        .to_string();
    (!query.is_empty()).then_some(())
}

pub fn list(conn: &Connection) -> Result<Vec<RecentSite>, String> {
    let mut found = Vec::new();
    let mut seen = HashSet::new();
    let mut cursor = None;
    loop {
        let rows = search_history(conn, 200, "", cursor.as_ref())?;
        for row in &rows {
            if let Some(mut site) = from_url(&row.url, &row.title) {
                if seen.insert(site.site_key.clone()) {
                    site.icon = site_icons_repo::data_url(conn, &site.site_key);
                    found.push(site);
                    if found.len() == 5 {
                        return Ok(found);
                    }
                }
            }
        }
        if rows.len() < 200 {
            return Ok(found);
        }
        cursor = rows.last().map(|row| HistoryCursor {
            visited_at: row.visited_at,
            id: row.id,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn excludes_only_known_search_results_and_internal_pages() {
        for raw in [
            "https://www.google.com.br/search?q=Rust+%26+Svelte",
            "https://duckduckgo.com/?q=x",
            "https://html.duckduckgo.com/html/?q=x",
            "https://lite.duckduckgo.com/lite/?q=x",
            "veloce://newtab",
            "https://user:pass@example.test/",
            "http://tauri.localhost/new-tab.html",
        ] {
            assert!(from_url(raw, "").is_none(), "{raw}");
        }
        for raw in [
            "https://www.google.com/",
            "https://google.com/search",
            "https://duckduckgo.com/about",
            "https://google.com.evil/search?q=x",
        ] {
            assert!(from_url(raw, "").is_some(), "{raw}");
        }
    }
    #[test]
    fn scans_old_visits_groups_sites_and_keeps_latest_full_destination() {
        let db = Connection::open_in_memory().unwrap();
        db.execute_batch(
            "CREATE TABLE history(id INTEGER PRIMARY KEY,title TEXT,url TEXT,visited_at INTEGER)",
        )
        .unwrap();
        for id in 1..=410 {
            db.execute(
                "INSERT INTO history VALUES(?1,'Search','https://google.com/search?q=x',?1)",
                [id],
            )
            .unwrap();
        }
        for (id, url) in [
            (1, "http://www.one.test/old"),
            (2, "https://one.test/new?q=a%26b#section"),
            (3, "https://two.test/path"),
            (4, "https://two.test:8443/port"),
            (5, "https://three.test/"),
            (6, "https://four.test/"),
        ] {
            db.execute(
                "UPDATE history SET url=?1,title=?1 WHERE id=?2",
                rusqlite::params![url, id],
            )
            .unwrap();
        }
        // Cache failure must not make the history list fail.
        let sites = list(&db).unwrap();
        assert_eq!(sites.len(), 5);
        assert_eq!(sites[3].site_key, "two.test");
        assert_eq!(sites[4].url, "https://one.test/new?q=a%26b#section");
        assert!(sites.iter().all(|s| s.icon.is_none()));
        db.execute("DELETE FROM history", []).unwrap();
        assert!(list(&db).unwrap().is_empty());
    }
}
