use rusqlite::{functions::FunctionFlags, Connection};
use serde::Serialize;
use unicode_normalization::{char::is_combining_mark, UnicodeNormalization};

pub fn fold(value: &str) -> String {
    value
        .nfd()
        .filter(|c| !is_combining_mark(*c))
        .flat_map(char::to_lowercase)
        .collect()
}
pub fn register(conn: &Connection) -> Result<(), String> {
    conn.create_scalar_function(
        "veloce_search_fold",
        1,
        FunctionFlags::SQLITE_UTF8 | FunctionFlags::SQLITE_DETERMINISTIC,
        |ctx| Ok(fold(&ctx.get::<String>(0)?)),
    )
    .map_err(|e| e.to_string())?;
    super::search_index::migrate(conn)
}
#[derive(Serialize)]
pub struct NoteSearchHit {
    pub url: String,
    pub title: String,
    pub preview: String,
    pub characters: usize,
}
#[derive(Serialize)]
pub struct SessionSearchHit {
    pub id: i64,
    pub name: String,
    pub preview: String,
    pub page_count: usize,
}
#[derive(Default, Serialize)]
pub struct CommandSearchResult {
    pub notes: Vec<NoteSearchHit>,
    pub sessions: Vec<SessionSearchHit>,
    pub notes_count: usize,
    pub sessions_count: usize,
    pub notes_has_more: bool,
    pub sessions_has_more: bool,
}
/// Limit IPC to two short previews per domain. Bound parameters treat '%' and '_' literally.
pub fn search(conn: &Connection, query: &str) -> Result<CommandSearchResult, String> {
    let query = fold(query.trim());
    if query.chars().count() > 256 {
        return Err("command_search.invalid_query".into());
    }
    let mut out = CommandSearchResult::default();
    if query.is_empty() {
        return Ok(out);
    }
    let pattern = super::search_index::pattern(&query);
    let note_candidates = if pattern.is_some() {
        "SELECT url FROM note_search WHERE note_search MATCH ?1"
    } else {
        "SELECT url FROM note_search WHERE instr(title,?1)>0 OR instr(address,?1)>0 OR instr(body,?1)>0"
    };
    let needle = pattern.as_deref().unwrap_or(&query);
    let sql = format!("SELECT url,substr(title,1,256),substr(text,1,160),length(text) FROM page_notes WHERE url IN ({note_candidates}) ORDER BY updated_at DESC,url LIMIT 3");
    let mut notes = conn.prepare(&sql).map_err(|e| e.to_string())?;
    out.notes = notes
        .query_map([needle], |r| {
            Ok(NoteSearchHit {
                url: r.get(0)?,
                title: r.get(1)?,
                preview: r.get(2)?,
                characters: r.get(3)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    out.notes_has_more = out.notes.len() > 2;
    out.notes.truncate(2);
    out.notes_count = out.notes.len();
    let candidates = if pattern.is_some() {
        "SELECT id FROM session_search WHERE session_search MATCH ?1 UNION SELECT session_id FROM session_page_search WHERE session_page_search MATCH ?1"
    } else {
        "SELECT id FROM session_search WHERE instr(name,?1)>0 UNION SELECT session_id FROM session_page_search WHERE instr(title,?1)>0 OR instr(address,?1)>0"
    };
    let sql = format!("SELECT s.id,s.name,
        (SELECT substr(title || ' · ' || url,1,160) FROM saved_session_pages WHERE session_id=s.id ORDER BY position LIMIT 1),
        (SELECT count(*) FROM saved_session_pages WHERE session_id=s.id)
        FROM saved_sessions s WHERE s.id IN ({candidates}) ORDER BY s.id DESC LIMIT 3");
    let mut sessions = conn.prepare(&sql).map_err(|e| e.to_string())?;
    out.sessions = sessions
        .query_map([needle], |r| {
            Ok(SessionSearchHit {
                id: r.get(0)?,
                name: r.get(1)?,
                preview: r.get::<_, Option<String>>(2)?.unwrap_or_default(),
                page_count: r.get(3)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    out.sessions_has_more = out.sessions.len() > 2;
    out.sessions.truncate(2);
    out.sessions_count = out.sessions.len();
    Ok(out)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn searches_unicode_literals_and_bounds_previews_without_returning_documents() {
        let c = Connection::open_in_memory().unwrap();
        crate::database::connection::initialize_schema(&c).unwrap();
        let text = format!("CAFÉ 100%_ <b> {}", "x".repeat(400));
        for i in 0..4 {
            crate::database::notes_repo::save(
                &c,
                &format!("https://example.test/{i}"),
                "Résumé",
                &text,
            )
            .unwrap();
        }
        let tab = crate::state::tab_model::TabInfo::new_default(
            "a".into(),
            "https://example.test/Caf%C3%A9".into(),
            true,
        );
        crate::database::saved_sessions_repo::save(&c, None, "Estúdio Café", &[tab]).unwrap();
        let hits = search(&c, "CAFE").unwrap();
        assert_eq!(hits.notes_count, 2);
        assert_eq!(hits.notes.len(), 2);
        assert!(hits.notes_has_more);
        assert_eq!(hits.sessions_count, 1);
        assert!(hits.notes[0].preview.chars().count() <= 160);
        assert!(hits.notes[0].characters > 400);
        assert_eq!(search(&c, "100%_").unwrap().notes_count, 2);
        assert_eq!(search(&c, "not found").unwrap().notes_count, 0);
        assert_eq!(search(&c, "resume").unwrap().notes_count, 2);
        assert!(search(&c, &"x".repeat(257)).is_err());
    }
}
