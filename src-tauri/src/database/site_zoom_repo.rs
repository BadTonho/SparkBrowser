use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct SiteZoom {
    pub origin: String,
    pub factor: f64,
}
pub fn origin(raw: &str) -> Result<String, String> {
    let url = url::Url::parse(raw).map_err(|_| "zoom.invalid_origin")?;
    if !matches!(url.scheme(), "http" | "https")
        || url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
    {
        return Err("zoom.invalid_origin".into());
    }
    Ok(url.origin().ascii_serialization())
}
pub fn validate(row: &SiteZoom) -> Result<(), String> {
    if origin(&row.origin)? != row.origin
        || !row.factor.is_finite()
        || !(0.25..=5.0).contains(&row.factor)
    {
        return Err("zoom.invalid_value".into());
    }
    Ok(())
}
pub fn migrate(conn: &Connection) -> Result<(), String> {
    super::migrations::apply(conn, "site-zoom-v1", |c| {
        c.execute_batch("CREATE TABLE site_zoom(origin TEXT PRIMARY KEY,factor REAL NOT NULL CHECK(factor BETWEEN 0.25 AND 5.0))").map_err(|e|e.to_string())
    })
}
pub fn list(conn: &Connection) -> Result<Vec<SiteZoom>, String> {
    conn.prepare("SELECT origin,factor FROM site_zoom ORDER BY origin")
        .map_err(|e| e.to_string())?
        .query_map([], |r| {
            Ok(SiteZoom {
                origin: r.get(0)?,
                factor: r.get(1)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}
pub fn set(conn: &Connection, row: &SiteZoom) -> Result<(), String> {
    validate(row)?;
    if (row.factor-1.0).abs()<0.0001 { conn.execute("DELETE FROM site_zoom WHERE origin=?1",[&row.origin]) }
    else { conn.execute("INSERT INTO site_zoom VALUES(?1,?2) ON CONFLICT(origin) DO UPDATE SET factor=excluded.factor",params![row.origin,row.factor]) }.map_err(|e|e.to_string())?;
    Ok(())
}
pub fn merge(conn: &Connection, rows: &[SiteZoom]) -> Result<(), String> {
    for row in rows {
        validate(row)?;
        conn.execute(
            "INSERT OR IGNORE INTO site_zoom VALUES(?1,?2)",
            params![row.origin, row.factor],
        )
        .map_err(|e| e.to_string())?;
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn origins_ranges_and_import_conflicts_preserve_local_preferences() {
        let c = Connection::open_in_memory().unwrap();
        migrate(&c).unwrap();
        assert_eq!(
            origin("HTTPS://EXAMPLE.test:443/a?q=1#b").unwrap(),
            "https://example.test"
        );
        assert_ne!(
            origin("http://example.test").unwrap(),
            origin("https://example.test").unwrap()
        );
        assert_ne!(
            origin("https://example.test:444").unwrap(),
            origin("https://example.test").unwrap()
        );
        let local = SiteZoom {
            origin: "https://example.test".into(),
            factor: 1.5,
        };
        set(&c, &local).unwrap();
        merge(
            &c,
            &[SiteZoom {
                factor: 2.0,
                ..local.clone()
            }],
        )
        .unwrap();
        assert_eq!(list(&c).unwrap(), vec![local.clone()]);
        assert!(set(
            &c,
            &SiteZoom {
                factor: 5.1,
                ..local.clone()
            }
        )
        .is_err());
        assert!(origin("file:///tmp/test").is_err());
        set(
            &c,
            &SiteZoom {
                factor: 1.0,
                ..local
            },
        )
        .unwrap();
        assert!(list(&c).unwrap().is_empty());
    }
}
