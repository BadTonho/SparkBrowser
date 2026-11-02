//! Explicit synthetic benchmark; timings are evidence, never pass/fail thresholds.
use super::{
    command_search_repo::{fold, search},
    connection::initialize_schema,
};
use rusqlite::{params, Connection};
use std::time::Instant;

#[test]
#[ignore = "synthetic search benchmark; run explicitly with --ignored --nocapture"]
fn compare_indexed_search_with_review_baseline() {
    let conn = Connection::open_in_memory().unwrap();
    initialize_schema(&conn).unwrap();
    let started = Instant::now();
    let tx = conn.unchecked_transaction().unwrap();
    for i in 0..5_000 {
        let body = format!(
            "Résumé common 100%_ {} needle-{i}",
            "local text ".repeat(100)
        );
        tx.execute(
            "INSERT INTO page_notes VALUES(?1,?2,?3,?4)",
            params![
                format!("https://example.test/{i}"),
                format!("Note {i}"),
                body,
                i
            ],
        )
        .unwrap();
    }
    for i in 0..1_000 {
        tx.execute(
            "INSERT INTO saved_sessions VALUES(?1,?2)",
            params![i, format!("Session résumé {i}")],
        )
        .unwrap();
        for position in 0..3 {
            tx.execute(
                "INSERT INTO saved_session_pages VALUES(?1,?2,?3,?4,0,0)",
                params![
                    i,
                    position,
                    "Common page",
                    format!("https://example.test/needle-{i}/{position}")
                ],
            )
            .unwrap();
        }
    }
    tx.commit().unwrap();
    println!("seed_ms={:.3}", started.elapsed().as_secs_f64() * 1_000.0);
    // SQL from the reviewed revision 1ba55b7, including per-row folding/exact totals.
    let mut old_notes = conn.prepare("SELECT url,substr(title,1,256),substr(text,1,160),length(text),count(*) OVER() FROM page_notes
        WHERE instr(veloce_search_fold(title),?1)>0 OR instr(veloce_search_fold(url),?1)>0 OR instr(veloce_search_fold(text),?1)>0
        ORDER BY updated_at DESC,url LIMIT 2").unwrap();
    let mut old_sessions = conn.prepare("SELECT s.id,s.name,
        (SELECT substr(title || ' · ' || url,1,160) FROM saved_session_pages WHERE session_id=s.id ORDER BY position LIMIT 1),
        (SELECT count(*) FROM saved_session_pages WHERE session_id=s.id),count(*) OVER() FROM saved_sessions s
        WHERE instr(veloce_search_fold(s.name),?1)>0 OR EXISTS(SELECT 1 FROM saved_session_pages p WHERE p.session_id=s.id AND
          (instr(veloce_search_fold(p.title),?1)>0 OR instr(veloce_search_fold(p.url),?1)>0)) ORDER BY s.id DESC LIMIT 2").unwrap();
    for query in [
        "needle-4999",
        "common",
        "résumé",
        "100%_",
        "no-match-absent",
        "x",
    ] {
        let needle = fold(query);
        let mut timings = Vec::new();
        for iteration in 0..6 {
            let start = Instant::now();
            let notes = old_notes
                .query_map([&needle], |r| r.get::<_, String>(0))
                .unwrap()
                .collect::<Result<Vec<_>, _>>()
                .unwrap();
            let sessions = old_sessions
                .query_map([&needle], |r| r.get::<_, i64>(0))
                .unwrap()
                .collect::<Result<Vec<_>, _>>()
                .unwrap();
            let baseline_ms = start.elapsed().as_secs_f64() * 1_000.0;
            let start = Instant::now();
            let current = search(&conn, query).unwrap();
            let indexed_ms = start.elapsed().as_secs_f64() * 1_000.0;
            assert_eq!(
                current
                    .notes
                    .iter()
                    .map(|n| n.url.clone())
                    .collect::<Vec<_>>(),
                notes
            );
            assert_eq!(
                current.sessions.iter().map(|s| s.id).collect::<Vec<_>>(),
                sessions
            );
            if iteration > 0 {
                timings.push(
                    serde_json::json!({"baseline_ms": baseline_ms, "indexed_ms": indexed_ms}),
                );
            }
        }
        println!(
            "{}",
            serde_json::json!({"query": query, "samples": timings})
        );
    }
    let index_bytes: i64 = conn
        .query_row(
            "SELECT sum(pgsize) FROM dbstat WHERE name LIKE '%search%'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    println!("index_bytes={index_bytes}");
}
