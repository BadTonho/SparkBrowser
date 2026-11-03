//! Rebuildable, bounded local favicon cache. Never exported in backups.
use base64::{engine::general_purpose::STANDARD, Engine};
use rusqlite::{params, Connection, OptionalExtension};

pub const MAX_BYTES: usize = 64 * 1024;
pub const MAX_ICONS: usize = 512;

pub fn migrate(conn: &Connection) -> Result<(), String> {
    super::migrations::apply(conn, "site-icons-v1", |conn| {
        conn.execute_batch("CREATE TABLE site_icons(site_key TEXT PRIMARY KEY, png BLOB NOT NULL, updated_at INTEGER NOT NULL)").map_err(|e| e.to_string())
    })
}

pub fn site_key(raw: &str) -> Option<String> {
    let url = url::Url::parse(raw).ok()?;
    if !matches!(url.scheme(), "http" | "https")
        || !url.username().is_empty()
        || url.password().is_some()
    {
        return None;
    }
    let hostname = url.host_str()?;
    let host = hostname.strip_prefix("www.").unwrap_or(hostname);
    if host.is_empty() || host == "tauri.localhost" {
        return None;
    }
    Some(match url.port() {
        Some(port) => format!("{host}:{port}"),
        None => host.into(),
    })
}

pub fn valid_png(bytes: &[u8]) -> bool {
    if bytes.len() > MAX_BYTES || bytes.is_empty() {
        return false;
    }
    let mut decoder = png::Decoder::new(std::io::Cursor::new(bytes));
    decoder.set_limits(png::Limits { bytes: 1024 * 1024 });
    let Ok(mut reader) = decoder.read_info() else {
        return false;
    };
    let info = reader.info();
    if info.width == 0 || info.height == 0 || info.width > 256 || info.height > 256 {
        return false;
    }
    let mut output = vec![0; reader.output_buffer_size()];
    reader.next_frame(&mut output).is_ok() && reader.finish().is_ok()
}

pub fn store(conn: &Connection, key: &str, bytes: &[u8]) -> Result<bool, String> {
    if !valid_png(bytes) {
        return Ok(false);
    }
    let tx = conn.unchecked_transaction().map_err(|e| e.to_string())?;
    let existing: Option<Vec<u8>> = tx
        .query_row(
            "SELECT png FROM site_icons WHERE site_key=?1 AND length(png)<=65536",
            [key],
            |r| r.get(0),
        )
        .optional()
        .map_err(|e| e.to_string())?;
    if existing.as_deref() == Some(bytes) {
        return Ok(false);
    }
    tx.execute("INSERT INTO site_icons(site_key,png,updated_at) VALUES(?1,?2,(SELECT COALESCE(MAX(updated_at),0)+1 FROM site_icons)) ON CONFLICT(site_key) DO UPDATE SET png=excluded.png,updated_at=excluded.updated_at",params![key,bytes]).map_err(|e|e.to_string())?;
    tx.execute("DELETE FROM site_icons WHERE site_key NOT IN (SELECT site_key FROM site_icons ORDER BY updated_at DESC,site_key LIMIT ?1)",[MAX_ICONS]).map_err(|e|e.to_string())?;
    tx.commit().map_err(|e| e.to_string())?;
    Ok(true)
}

pub fn store_current(
    conn: &Connection,
    key: &str,
    bytes: &[u8],
    captured_epoch: u64,
    current_epoch: u64,
) -> Result<bool, String> {
    if captured_epoch != current_epoch {
        return Ok(false);
    }
    store(conn, key, bytes)
}

pub fn data_url(conn: &Connection, key: &str) -> Option<String> {
    let bytes: Vec<u8> = conn
        .query_row("SELECT png FROM site_icons WHERE site_key=?1", [key], |r| {
            r.get(0)
        })
        .ok()?;
    valid_png(&bytes).then(|| format!("data:image/png;base64,{}", STANDARD.encode(bytes)))
}

pub fn clear(conn: &Connection) -> Result<(), String> {
    conn.execute("DELETE FROM site_icons", [])
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    pub fn png() -> Vec<u8> {
        let mut out = Vec::new();
        {
            let mut encoder = png::Encoder::new(&mut out, 1, 1);
            encoder.set_color(png::ColorType::Rgba);
            let mut writer = encoder.write_header().unwrap();
            writer.write_image_data(&[70, 100, 200, 255]).unwrap();
        }
        out
    }
    #[test]
    fn cache_migration_limits_validation_and_clear() {
        let db = Connection::open_in_memory().unwrap();
        migrate(&db).unwrap();
        migrate(&db).unwrap();
        let bytes = png();
        assert!(valid_png(&bytes));
        assert!(!valid_png(b"not a png"));
        assert!(!valid_png(&bytes[..bytes.len() - 8]));
        assert!(!valid_png(&vec![0; MAX_BYTES + 1]));
        for n in 0..514 {
            assert!(store(&db, &format!("{n}.test"), &bytes).unwrap());
        }
        assert_eq!(
            db.query_row("SELECT COUNT(*) FROM site_icons", [], |r| r
                .get::<_, usize>(0))
                .unwrap(),
            512
        );
        assert!(data_url(&db, "0.test").is_none());
        assert!(data_url(&db, "513.test")
            .unwrap()
            .starts_with("data:image/png;base64,"));
        assert!(!store(&db, "513.test", &bytes).unwrap());
        db.execute(
            "UPDATE site_icons SET png=x'00' WHERE site_key='513.test'",
            [],
        )
        .unwrap();
        assert!(data_url(&db, "513.test").is_none());
        clear(&db).unwrap();
        assert!(data_url(&db, "512.test").is_none());
    }
    #[test]
    fn groups_schemes_www_and_preserves_unusual_ports() {
        for url in [
            "http://www.example.test/a",
            "https://example.test:443/b",
            "http://example.test:80",
        ] {
            assert_eq!(site_key(url).as_deref(), Some("example.test"));
        }
        assert_eq!(
            site_key("https://example.test:444/a").as_deref(),
            Some("example.test:444")
        );
        for raw in [
            "veloce://newtab",
            "file:///x",
            "https://user:pass@example.test",
            "http://tauri.localhost/new-tab.html",
        ] {
            assert!(site_key(raw).is_none());
        }
    }
    #[test]
    fn clearing_rejects_old_captures_and_failed_writes_leave_cache_retryable() {
        let db = Connection::open_in_memory().unwrap();
        migrate(&db).unwrap();
        let png = png();
        assert!(store_current(&db, "kept.test", &png, 0, 0).unwrap());
        clear(&db).unwrap();
        assert!(!store_current(&db, "kept.test", &png, 0, 1).unwrap());
        assert!(data_url(&db, "kept.test").is_none());
        db.execute_batch("CREATE TRIGGER fail_icon BEFORE INSERT ON site_icons BEGIN SELECT RAISE(ABORT,'injected'); END").unwrap();
        assert!(store_current(&db, "new.test", &png, 1, 1).is_err());
        db.execute_batch("DROP TRIGGER fail_icon").unwrap();
        assert!(store_current(&db, "new.test", &png, 1, 1).unwrap());
    }
}
