use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
#[derive(Clone, Serialize, Deserialize)]
pub struct SiteSearchShortcut {
    pub keyword: String,
    pub name: String,
    pub template: String,
}
pub fn migrate(conn: &Connection) -> Result<(), String> {
    super::migrations::apply(conn, "site-search-v1", |conn| {
        conn.execute_batch("CREATE TABLE site_search_shortcuts(keyword TEXT PRIMARY KEY COLLATE NOCASE,name TEXT NOT NULL,template TEXT NOT NULL)").map_err(|e| e.to_string())
    })
}
pub fn validate(item: &SiteSearchShortcut) -> Result<(), String> {
    if item.keyword.is_empty()
        || item.keyword.len() > 24
        || !item
            .keyword
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
    {
        return Err("Keyword must contain 1–24 letters, digits, _ or -".into());
    }
    if item.name.trim().is_empty() || item.name.chars().count() > 100 {
        return Err("Name must contain 1–100 characters".into());
    }
    if item.template.len() > 4096 || item.template.matches("{query}").count() != 1 {
        return Err("Template must contain exactly one {query}".into());
    }
    let url = url::Url::parse(&item.template.replace("{query}", "VELOCEQUERYMARKER"))
        .map_err(|_| "Invalid search URL")?;
    let prefix = item
        .template
        .split_once("{query}")
        .expect("validated marker")
        .0;
    if !matches!(url.scheme(), "http" | "https")
        || url.host_str().is_none()
        || prefix.contains('#')
        || url
            .host_str()
            .is_some_and(|host| host.contains("velocequerymarker"))
        || !url.username().is_empty()
        || url.password().is_some()
        || !(url.path().contains("VELOCEQUERYMARKER")
            || url.query().is_some_and(|q| q.contains("VELOCEQUERYMARKER")))
    {
        return Err("Use an HTTP/HTTPS template with {query} in its path or query".into());
    }
    Ok(())
}
pub fn list(conn: &Connection) -> Result<Vec<SiteSearchShortcut>, String> {
    let mut stmt = conn
        .prepare_cached("SELECT keyword,name,template FROM site_search_shortcuts ORDER BY keyword")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |r| {
            Ok(SiteSearchShortcut {
                keyword: r.get(0)?,
                name: r.get(1)?,
                template: r.get(2)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    Ok(rows)
}
pub fn save(
    conn: &Connection,
    previous: Option<&str>,
    item: &SiteSearchShortcut,
) -> Result<(), String> {
    validate(item)?;
    let tx = conn.unchecked_transaction().map_err(|e| e.to_string())?;
    if let Some(previous) = previous {
        tx.execute(
            "DELETE FROM site_search_shortcuts WHERE keyword=?1",
            [previous],
        )
        .map_err(|e| e.to_string())?;
    }
    tx.execute(
        "INSERT INTO site_search_shortcuts VALUES(?1,?2,?3)",
        params![
            item.keyword.to_ascii_lowercase(),
            item.name.trim(),
            item.template
        ],
    )
    .map_err(|_| "Keyword already exists or could not be saved".to_string())?;
    tx.commit().map_err(|e| e.to_string())
}
pub fn resolve(conn: &Connection, input: &str) -> Result<Option<url::Url>, String> {
    let Some((keyword, query)) = input.trim().split_once(char::is_whitespace) else {
        return Ok(None);
    };
    let query = query.trim();
    if query.is_empty() {
        return Ok(None);
    }
    let template: Option<String> = conn
        .query_row(
            "SELECT template FROM site_search_shortcuts WHERE keyword=?1",
            [keyword],
            |r| r.get(0),
        )
        .optional()
        .map_err(|e| e.to_string())?;
    template
        .map(|template| {
            let encoded = url::form_urlencoded::byte_serialize(query.as_bytes())
                .collect::<String>()
                .replace('+', "%20");
            url::Url::parse(&template.replace("{query}", &encoded)).map_err(|e| e.to_string())
        })
        .transpose()
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn validates_and_resolves_without_double_encoding_or_losing_previous_edits() {
        let conn = Connection::open_in_memory().unwrap();
        migrate(&conn).unwrap();
        let mut item = SiteSearchShortcut {
            keyword: "gh".into(),
            name: "GitHub".into(),
            template: "https://github.com/search?q={query}".into(),
        };
        save(&conn, None, &item).unwrap();
        let url = resolve(&conn, "GH ação & + /%").unwrap().unwrap();
        assert_eq!(url.query_pairs().next().unwrap().1, "ação & + /%");
        assert!(resolve(&conn, "missing query").unwrap().is_none());
        assert!(resolve(&conn, "gh").unwrap().is_none());
        item.keyword = "yt".into();
        save(&conn, None, &item).unwrap();
        assert!(save(&conn, Some("gh"), &item).is_err());
        assert_eq!(list(&conn).unwrap().len(), 2);
        for template in [
            "https://{query}.test/",
            "https://example.test/#{query}",
            "javascript:{query}",
            "https://example.test/?q={query}&x={query}",
        ] {
            item.template = template.into();
            assert!(validate(&item).is_err());
        }
    }
}
