//! Data for the suggestion list of the new tab search field. The ranking is the address bar's own code,
//! which runs in the page (`public/new-tab/suggestions-core.js`); this side only reads the local data
//! and, as a separate step, asks the search engine for terms.
use super::*;
use crate::database::{bookmarks_repo, executor, history_repo, site_icons_repo};
use rusqlite::Connection;
use serde::Serialize;
use std::collections::{BTreeMap, HashMap};

/// The same amount the address bar reads.
const HISTORY_ROWS: usize = 60;
const BOOKMARK_ROWS: usize = 8;

#[derive(Serialize)]
struct Bookmark {
    title: String,
    url: String,
}

#[derive(Serialize)]
struct Local {
    history: Vec<history_repo::HistoryEntry>,
    bookmarks: Vec<Bookmark>,
    /// Cached site icons, keyed by site.
    icons: BTreeMap<String, String>,
    /// Which cached icon each listed address uses.
    sites: BTreeMap<String, String>,
}

fn local(conn: &Connection, query: &str) -> Result<Local, String> {
    let query = query.trim();
    let history = history_repo::search_history(conn, HISTORY_ROWS, query, None)?;
    let words: Vec<String> = query.split_whitespace().map(str::to_lowercase).collect();
    // With nothing typed the list is only the recent sites.
    let bookmarks: Vec<Bookmark> = if words.is_empty() {
        Vec::new()
    } else {
        bookmarks_repo::list_bookmarks(conn)?
            .into_iter()
            .filter(|item| {
                let text = format!("{} {}", item.title, item.url).to_lowercase();
                words.iter().all(|word| text.contains(word.as_str()))
            })
            .take(BOOKMARK_ROWS)
            .map(|item| Bookmark {
                title: item.title,
                url: item.url,
            })
            .collect()
    };
    let mut icons = BTreeMap::new();
    let mut sites = BTreeMap::new();
    let mut looked_up = HashMap::new();
    let addresses = bookmarks
        .iter()
        .map(|item| &item.url)
        .chain(history.iter().map(|row| &row.url));
    for url in addresses {
        let Some(key) = site_icons_repo::site_key(url) else {
            continue;
        };
        let found = *looked_up.entry(key.clone()).or_insert_with(|| {
            site_icons_repo::data_url(conn, &key)
                .map(|icon| icons.insert(key.clone(), icon))
                .is_some()
        });
        if found {
            sites.insert(url.clone(), key);
        }
    }
    Ok(Local {
        history,
        bookmarks,
        icons,
        sites,
    })
}

/// `/suggest` answers with local history and bookmarks; `/terms` with the search engine's proposals.
pub(super) fn handle(
    app: &AppHandle,
    id: &str,
    label: &str,
    token: &str,
    document: String,
    values: &HashMap<String, String>,
    terms: bool,
) {
    let Some(request) = values.get("request").and_then(|v| v.parse::<u64>().ok()) else {
        return;
    };
    let query = values.get("input").cloned().unwrap_or_default();
    let (app, id, label, token) = (
        app.clone(),
        id.to_owned(),
        label.to_owned(),
        token.to_owned(),
    );
    tauri::async_runtime::spawn(async move {
        let epoch = super::super::favicon::epoch(&app);
        let payload = if terms {
            // What is typed never leaves the computer early from a private window.
            if app
                .try_state::<crate::private_session::PrivateRole>()
                .is_some_and(|role| role.is_private())
            {
                return;
            }
            let Ok(engine) = app
                .state::<NewTabService>()
                .preferences
                .lock()
                .map(|p| p.engine.clone())
            else {
                return;
            };
            match crate::commands::search_suggestions::fetch_search_suggestions(engine, query).await
            {
                Ok(found) => serde_json::json!({"request": request, "terms": found}),
                Err(error) => {
                    crate::error_log::warn(&format!("New tab search terms failed: {error}"));
                    return;
                }
            }
        } else {
            match executor::run(app.clone(), move |conn| local(conn, &query)).await {
                Ok(found) => serde_json::json!({"request": request, "local": found}),
                Err(error) => {
                    crate::error_log::warn(&format!(
                        "New tab suggestions could not be loaded: {error}"
                    ));
                    return;
                }
            }
        };
        // The page may have navigated, or the history been cleared, while this was read.
        if !super::bridge::valid(&app, &id, &label, &token)
            || super::super::favicon::epoch(&app) != epoch
        {
            return;
        }
        if let Some(view) = app.get_webview(&label) {
            crate::error_log::ignored(view.eval(format!(
                "window.__sparkSuggestReply?.({}, {});",
                serde_json::Value::from(document),
                payload
            )));
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn database() -> Connection {
        let db = Connection::open_in_memory().unwrap();
        db.execute_batch(
            "CREATE TABLE history(id INTEGER PRIMARY KEY,title TEXT,url TEXT,visited_at INTEGER);
             CREATE TABLE bookmarks(id INTEGER PRIMARY KEY,title TEXT,url TEXT,icon TEXT,folder_id INTEGER);
             CREATE TABLE site_icons(site_key TEXT PRIMARY KEY, png BLOB NOT NULL, updated_at INTEGER NOT NULL);
             INSERT INTO history VALUES(1,'Rust book','https://doc.rust-lang.org/book/',1),(2,'Cooking','https://food.test/',2);
             INSERT INTO bookmarks VALUES(1,'Rust playground','https://play.rust-lang.org/','',NULL),(2,'News','https://news.test/','',NULL);",
        )
        .unwrap();
        db
    }

    #[test]
    fn typed_words_filter_history_and_bookmarks() {
        let found = local(&database(), " rust ").unwrap();
        assert_eq!(found.history.len(), 1);
        assert_eq!(found.bookmarks.len(), 1);
        assert_eq!(found.bookmarks[0].title, "Rust playground");
        assert!(found.icons.is_empty() && found.sites.is_empty());
        assert!(local(&database(), "rust news")
            .unwrap()
            .bookmarks
            .is_empty());
    }

    #[test]
    fn an_empty_field_offers_recent_history_and_no_bookmarks() {
        let found = local(&database(), "").unwrap();
        assert_eq!(found.history.len(), 2);
        assert_eq!(found.history[0].title, "Cooking");
        assert!(found.bookmarks.is_empty());
    }
}
