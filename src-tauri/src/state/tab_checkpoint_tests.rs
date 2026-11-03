use super::*;
use crate::database::{connection::initialize_schema, session_repo::load_session_tabs};
use rusqlite::Connection;
use std::sync::atomic::Ordering;

fn state() -> BrowserState {
    let state = BrowserState::default();
    let conn = Connection::open_in_memory().unwrap();
    initialize_schema(&conn).unwrap();
    *state.db.lock().unwrap() = Some(conn);
    for id in ["tab-2", "tab-3"] {
        state.tabs.lock().unwrap().push(TabInfo::new_default(
            id.into(),
            format!("https://example.test/{id}"),
            false,
        ));
    }
    state.persist_session().unwrap();
    state
}
fn memory(state: &BrowserState) -> serde_json::Value {
    serde_json::json!({
        "tabs": state.snapshot_tabs(),
        "layouts": *state.split_layouts.lock().unwrap(),
        "tab": *state.active_tab_id.lock().unwrap(),
        "workspace": *state.active_workspace_id.lock().unwrap(),
        "limit": state.resident_limit.load(Ordering::Relaxed),
    })
}
fn disk(state: &BrowserState) -> serde_json::Value {
    serde_json::to_value(load_session_tabs(state.db.lock().unwrap().as_ref().unwrap()).unwrap())
        .unwrap()
}

#[test]
fn persistence_failures_restore_organization_selection_and_residency() {
    type Change = fn(&BrowserState) -> Result<(), String>;
    let changes: [(&str, Change); 6] = [
        ("reorder", |s| {
            s.tabs.lock().unwrap().reverse();
            Ok(())
        }),
        ("switch", |s| s.select_tab("tab-2").map(|_| ())),
        ("pin", |s| {
            s.tabs.lock().unwrap()[1].is_pinned = true;
            Ok(())
        }),
        ("move", |s| s.move_tab("tab-1", "ws-work")),
        ("limit", |s| {
            s.resident_limit.store(1, Ordering::Relaxed);
            Ok(())
        }),
        ("create", |s| {
            s.tabs.lock().unwrap().push(TabInfo::new_default(
                "tab-4".into(),
                "https://example.test/new".into(),
                false,
            ));
            s.select_tab("tab-4").map(|_| ())
        }),
    ];
    for (name, change) in changes {
        let state = state();
        let before = memory(&state);
        let saved = disk(&state);
        state.db.lock().unwrap().as_ref().unwrap().execute_batch(
            "CREATE TRIGGER fail_session BEFORE INSERT ON session_tabs BEGIN SELECT RAISE(ABORT,'injected session failure'); END;"
        ).unwrap();
        let error = TabCheckpoint::commit(&state, || change(&state)).unwrap_err();
        assert!(
            error.contains("injected session failure"),
            "{name}: {error}"
        );
        assert_eq!(memory(&state), before, "{name} changed memory");
        assert_eq!(disk(&state), saved, "{name} changed persisted tabs");
        state
            .db
            .lock()
            .unwrap()
            .as_ref()
            .unwrap()
            .execute_batch("DROP TRIGGER fail_session")
            .unwrap();
        TabCheckpoint::commit(&state, || change(&state)).unwrap();
        assert_ne!(
            memory(&state),
            before,
            "{name} must work after failure is removed"
        );
    }
}

#[test]
fn rollback_keeps_document_events_received_during_a_failed_change() {
    let state = state();
    let before = TabCheckpoint::capture(&state).unwrap();
    state.select_tab("tab-2").unwrap();
    {
        let mut tabs = state.tabs.lock().unwrap();
        tabs[0].url = "https://example.test/redirect".into();
        tabs[0].title = "Loaded during the operation".into();
        tabs[0].navigation.revision += 1;
        tabs[0].activity.native_audio = true;
        tabs[0].activity.refresh_media();
    }
    before.restore(&state).unwrap();
    let tabs = state.snapshot_tabs();
    assert_eq!(*state.active_tab_id.lock().unwrap(), "tab-1");
    assert_eq!(tabs[0].url, "https://example.test/redirect");
    assert_eq!(tabs[0].navigation.revision, 1);
    assert!(tabs[0].activity.playing_media);
}

#[test]
fn activation_and_session_changes_commit_together() {
    let state = state();
    let before = memory(&state);
    let saved = disk(&state);
    state.db.lock().unwrap().as_ref().unwrap().execute_batch(
        "CREATE TRIGGER fail_activation BEFORE INSERT ON tab_usage_events BEGIN SELECT RAISE(ABORT,'injected activation failure'); END;"
    ).unwrap();
    assert!(
        TabCheckpoint::commit_with_activation(&state, Some(("tab-2", false)), || state
            .select_tab("tab-2"))
        .is_err()
    );
    assert_eq!(memory(&state), before);
    assert_eq!(disk(&state), saved);
    let db = state.db.lock().unwrap();
    let conn = db.as_ref().unwrap();
    let count: i64 = conn
        .query_row("SELECT count(*) FROM tab_usage_events", [], |r| r.get(0))
        .unwrap();
    assert_eq!(count, 0);
}

#[test]
fn fallback_shield_metadata_is_restored_before_any_document_is_rebuilt() {
    let state = state();
    state.tabs.lock().unwrap()[0].url = "https://google-analytics.com/collect".into();
    state.persist_session().unwrap();
    let before = memory(&state);
    state.db.lock().unwrap().as_ref().unwrap().execute_batch(
        "CREATE TRIGGER fail_session BEFORE INSERT ON session_tabs BEGIN SELECT RAISE(ABORT,'injected reset failure'); END;"
    ).unwrap();
    assert!(TabCheckpoint::commit(&state, || {
        crate::window::webview_manager::prepare_content_reset(&state, true)
    })
    .is_err());
    assert_eq!(memory(&state), before);
}
