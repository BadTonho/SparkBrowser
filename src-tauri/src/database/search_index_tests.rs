use crate::database::{
    command_search_repo::{register, search},
    connection::initialize_schema,
    notes_repo, saved_sessions_repo,
};
use crate::state::tab_model::TabInfo;
use rusqlite::Connection;

#[test]
fn nul_queries_use_literal_matching_without_fts_parse_errors() {
    let conn = Connection::open_in_memory().unwrap();
    initialize_schema(&conn).unwrap();
    notes_repo::save(
        &conn,
        "https://example.test/nul",
        "Note",
        "before abc\0xyz after",
    )
    .unwrap();
    let mut tab = TabInfo::new_default("t".into(), "https://example.test/page".into(), true);
    tab.title = "abc\0xyz".into();
    saved_sessions_repo::save(&conn, None, "Session", &[tab]).unwrap();
    let result = search(&conn, "abc\0xyz").unwrap();
    assert_eq!(result.notes_count, 1);
    assert_eq!(result.sessions_count, 1);
    assert_eq!(search(&conn, "missing\0text").unwrap().notes_count, 0);
    assert_eq!(search(&conn, "\0").unwrap().notes_count, 1);
}

#[test]
fn index_matches_literal_substrings_and_tracks_updates_deletes_and_rollbacks() {
    let conn = Connection::open_in_memory().unwrap();
    initialize_schema(&conn).unwrap();
    let url = "https://example.test/note";
    let text = "Café Ω中 🦀 quoted \"literal\" 100%_ a_b x!y foo-bar AND résumé";
    notes_repo::save(&conn, url, "Title", text).unwrap();
    for query in [
        "cafe",
        "Ω",
        "Ω中",
        "🦀",
        "\"literal\"",
        "%_",
        "100%_",
        "a_b",
        "x!y",
        "foo-bar",
        "AND",
        "resume",
        "quoted \"literal\"",
    ] {
        assert_eq!(search(&conn, query).unwrap().notes_count, 1, "{query}");
    }
    assert_eq!(search(&conn, "100x_").unwrap().notes_count, 0);
    let tx = conn.unchecked_transaction().unwrap();
    notes_repo::save(&tx, url, "Title", "replacement").unwrap();
    assert_eq!(search(&tx, "cafe").unwrap().notes_count, 0);
    assert_eq!(search(&tx, "replacement").unwrap().notes_count, 1);
    tx.rollback().unwrap();
    assert_eq!(search(&conn, "cafe").unwrap().notes_count, 1);
    notes_repo::save(&conn, url, "Title", "replacement").unwrap();
    assert_eq!(search(&conn, "cafe").unwrap().notes_count, 0);
    notes_repo::delete(&conn, url).unwrap();
    assert_eq!(search(&conn, "replacement").unwrap().notes_count, 0);

    let tab = TabInfo::new_default("t".into(), "https://example.test/unique-page".into(), true);
    saved_sessions_repo::save(&conn, None, "Old café", &[tab]).unwrap();
    let id = saved_sessions_repo::list(&conn).unwrap()[0].id;
    assert_eq!(search(&conn, "unique-page").unwrap().sessions_count, 1);
    saved_sessions_repo::save(&conn, Some(id), "New résumé", &[]).unwrap_err();
    conn.execute(
        "UPDATE saved_sessions SET name='New résumé' WHERE id=?1",
        [id],
    )
    .unwrap();
    conn.execute(
        "UPDATE saved_session_pages SET url='https://example.test/changed' WHERE session_id=?1",
        [id],
    )
    .unwrap();
    assert_eq!(search(&conn, "old cafe").unwrap().sessions_count, 0);
    assert_eq!(search(&conn, "new resume").unwrap().sessions_count, 1);
    assert_eq!(search(&conn, "unique-page").unwrap().sessions_count, 0);
    conn.execute("DELETE FROM saved_session_pages WHERE session_id=?1", [id])
        .unwrap();
    conn.execute("DELETE FROM saved_sessions WHERE id=?1", [id])
        .unwrap();
    assert_eq!(search(&conn, "changed").unwrap().sessions_count, 0);
    assert_eq!(search(&conn, "new resume").unwrap().sessions_count, 0);
}

#[test]
fn migration_backfills_existing_data_once() {
    let conn = Connection::open_in_memory().unwrap();
    initialize_schema(&conn).unwrap();
    for table in ["note", "session", "session_page"] {
        for event in ["insert", "update", "delete"] {
            conn.execute_batch(&format!("DROP TRIGGER {table}_search_{event};"))
                .unwrap();
        }
        conn.execute_batch(&format!("DROP TABLE {table}_search;"))
            .unwrap();
    }
    conn.execute(
        "DELETE FROM app_migrations WHERE name='command-search-trigrams-v1'",
        [],
    )
    .unwrap();
    notes_repo::save(
        &conn,
        "https://example.test/old",
        "Preexisting résumé",
        "legacy data",
    )
    .unwrap();
    let tab = TabInfo::new_default("t".into(), "https://example.test/legacy".into(), true);
    saved_sessions_repo::save(&conn, None, "Legacy café", &[tab]).unwrap();
    register(&conn).unwrap();
    register(&conn).unwrap();
    assert_eq!(search(&conn, "resume").unwrap().notes_count, 1);
    assert_eq!(search(&conn, "legacy").unwrap().sessions_count, 1);
    let count: i64 = conn
        .query_row("SELECT count(*) FROM note_search", [], |r| r.get(0))
        .unwrap();
    assert_eq!(count, 1);
}
