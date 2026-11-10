use super::*;
use crate::database::{connection::initialize_schema, session_repo::load_session_tabs};
use rusqlite::Connection;

fn state() -> BrowserState {
    let state = BrowserState::default();
    let conn = Connection::open_in_memory().unwrap();
    initialize_schema(&conn).unwrap();
    *state.infrastructure().db().lock().unwrap() = Some(conn);
    for id in ["tab-2", "tab-3"] {
        state
            .session()
            .edit_tabs()
            .lock()
            .unwrap()
            .push(TabInfo::new_default(
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
        "layouts": *state.session().edit_split_layouts().lock().unwrap(),
        "tab": *state.session().edit_active_tab_id().lock().unwrap(),
        "workspace": *state.session().edit_active_workspace_id().lock().unwrap(),
        "limit": state.session().resident_limit(),
    })
}
fn disk(state: &BrowserState) -> serde_json::Value {
    let db = state.infrastructure().db().lock().unwrap();
    // Compare every persisted column, not a restored runtime object: restoration
    // assigns the current clock and hibernation defaults on every read.
    let mut statement = db.as_ref().unwrap().prepare(
        "SELECT tab_id,title,url,workspace_id,is_active,is_hibernated,position,is_pinned,container_id FROM session_tabs ORDER BY position,tab_id"
    ).unwrap();
    let rows = statement
        .query_map([], |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, String>(0)?,
                "title": row.get::<_, String>(1)?,
                "url": row.get::<_, String>(2)?,
                "workspace_id": row.get::<_, String>(3)?,
                "is_active": row.get::<_, bool>(4)?,
                "is_hibernated": row.get::<_, bool>(5)?,
                "position": row.get::<_, i64>(6)?,
                "is_pinned": row.get::<_, bool>(7)?,
                "container_id": row.get::<_, Option<String>>(8)?,
            }))
        })
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    serde_json::to_value(rows).unwrap()
}

#[test]
fn native_pane_selection_rolls_back_and_can_be_retried() {
    let state = state();
    let before = memory(&state);
    let saved = disk(&state);
    state.infrastructure().db().lock().unwrap().as_ref().unwrap().execute_batch(
        "CREATE TRIGGER fail_focus BEFORE INSERT ON session_tabs BEGIN SELECT RAISE(ABORT,'injected focus failure'); END;"
    ).unwrap();
    assert!(state
        .select_tab_committed("tab-2")
        .unwrap_err()
        .contains("injected focus failure"));
    assert_eq!(memory(&state), before);
    assert_eq!(disk(&state), saved);
    state
        .infrastructure()
        .db()
        .lock()
        .unwrap()
        .as_ref()
        .unwrap()
        .execute_batch("DROP TRIGGER fail_focus")
        .unwrap();
    state.select_tab_committed("tab-2").unwrap();
    assert_eq!(*state.session().active_tab_id().lock().unwrap(), "tab-2");
    assert!(load_session_tabs(
        state
            .infrastructure()
            .db()
            .lock()
            .unwrap()
            .as_ref()
            .unwrap()
    )
    .unwrap()
    .iter()
    .any(|tab| tab.id == "tab-2" && tab.is_active));
}

#[test]
fn persistence_failures_restore_organization_selection_and_residency() {
    type Change = fn(&BrowserState) -> Result<(), String>;
    let changes: [(&str, Change); 6] = [
        ("reorder", |s| {
            s.session().edit_tabs().lock().unwrap().reverse();
            Ok(())
        }),
        ("switch", |s| s.select_tab("tab-2").map(|_| ())),
        ("pin", |s| {
            s.session().edit_tabs().lock().unwrap()[1].is_pinned = true;
            Ok(())
        }),
        ("move", |s| s.move_tab("tab-1", "ws-work")),
        ("limit", |s| {
            s.session().set_resident_limit(1);
            Ok(())
        }),
        ("create", |s| {
            s.session()
                .edit_tabs()
                .lock()
                .unwrap()
                .push(TabInfo::new_default(
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
        state.infrastructure().db().lock().unwrap().as_ref().unwrap().execute_batch(
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
            .infrastructure()
            .db()
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
        let mut tabs = state.session().edit_tabs().lock().unwrap();
        tabs[0].url = "https://example.test/redirect".into();
        tabs[0].title = "Loaded during the operation".into();
        tabs[0].navigation.revision += 1;
        tabs[0].activity.native_audio = true;
        tabs[0].activity.refresh_media();
    }
    before.restore(&state).unwrap();
    let tabs = state.snapshot_tabs();
    assert_eq!(
        *state.session().edit_active_tab_id().lock().unwrap(),
        "tab-1"
    );
    assert_eq!(tabs[0].url, "https://example.test/redirect");
    assert_eq!(tabs[0].navigation.revision, 1);
    assert!(tabs[0].activity.playing_media);
}

#[test]
fn activation_and_session_changes_commit_together() {
    let state = state();
    let before = memory(&state);
    let saved = disk(&state);
    state.infrastructure().db().lock().unwrap().as_ref().unwrap().execute_batch(
        "CREATE TRIGGER fail_activation BEFORE INSERT ON tab_usage_events BEGIN SELECT RAISE(ABORT,'injected activation failure'); END;"
    ).unwrap();
    assert!(
        TabCheckpoint::commit_with_activation(&state, Some(("tab-2", false)), || state
            .select_tab("tab-2"))
        .is_err()
    );
    assert_eq!(memory(&state), before);
    assert_eq!(disk(&state), saved);
    let db = state.infrastructure().db().lock().unwrap();
    let conn = db.as_ref().unwrap();
    let count: i64 = conn
        .query_row("SELECT count(*) FROM tab_usage_events", [], |r| r.get(0))
        .unwrap();
    assert_eq!(count, 0);
}

#[test]
fn domain_side_effects_and_tab_changes_share_one_transaction_and_retry() {
    let state = state();
    state
        .infrastructure()
        .db()
        .lock()
        .unwrap()
        .as_ref()
        .unwrap()
        .execute_batch(
            "CREATE TABLE domain_marker(value INTEGER); INSERT INTO domain_marker VALUES(0)",
        )
        .unwrap();
    let before = memory(&state);
    let saved = disk(&state);
    let result = TabCheckpoint::commit_transaction(
        &state,
        None,
        || state.select_tab("tab-2"),
        |conn| {
            conn.execute("UPDATE domain_marker SET value=1", [])
                .unwrap();
            Err("injected domain failure".into())
        },
    );
    assert!(result.is_err());
    assert_eq!(memory(&state), before);
    assert_eq!(disk(&state), saved);
    let value: i64 = state
        .infrastructure()
        .db()
        .lock()
        .unwrap()
        .as_ref()
        .unwrap()
        .query_row("SELECT value FROM domain_marker", [], |r| r.get(0))
        .unwrap();
    assert_eq!(value, 0);
    TabCheckpoint::commit_transaction(
        &state,
        None,
        || state.select_tab("tab-2"),
        |conn| {
            conn.execute("UPDATE domain_marker SET value=1", [])
                .map_err(|e| e.to_string())?;
            Ok(())
        },
    )
    .unwrap();
    assert_eq!(
        state
            .snapshot_tabs()
            .iter()
            .find(|tab| tab.is_active)
            .unwrap()
            .id,
        "tab-2"
    );
    assert_eq!(disk(&state)[1]["is_active"], true);
}

#[test]
fn failed_policy_write_preserves_session_metadata_and_retries_without_a_reset() {
    let state = state();
    crate::database::preferences_repo::initialize_preferences_table(
        state
            .infrastructure()
            .db()
            .lock()
            .unwrap()
            .as_ref()
            .unwrap(),
    )
    .unwrap();
    let before = memory(&state);
    state.infrastructure().db().lock().unwrap().as_ref().unwrap().execute_batch(
        "CREATE TRIGGER fail_policy BEFORE INSERT ON app_preferences BEGIN SELECT RAISE(ABORT,'injected policy failure'); END"
    ).unwrap();
    assert!(state
        .commit_content_policy("fixture-policy", "true", true)
        .is_err());
    assert_eq!(memory(&state), before);
    state
        .infrastructure()
        .db()
        .lock()
        .unwrap()
        .as_ref()
        .unwrap()
        .execute_batch("DROP TRIGGER fail_policy")
        .unwrap();
    state
        .commit_content_policy("fixture-policy", "true", true)
        .unwrap();
    let value = crate::database::preferences_repo::load(
        state
            .infrastructure()
            .db()
            .lock()
            .unwrap()
            .as_ref()
            .unwrap(),
        "fixture-policy",
    )
    .unwrap();
    assert_eq!(value.as_deref(), Some("true"));
}

#[test]
fn fallback_shield_metadata_is_restored_before_any_document_is_rebuilt() {
    let state = state();
    state.session().edit_tabs().lock().unwrap()[0].url =
        "https://google-analytics.com/collect".into();
    state.persist_session().unwrap();
    let before = memory(&state);
    state.infrastructure().db().lock().unwrap().as_ref().unwrap().execute_batch(
        "CREATE TRIGGER fail_session BEFORE INSERT ON session_tabs BEGIN SELECT RAISE(ABORT,'injected reset failure'); END;"
    ).unwrap();
    assert!(TabCheckpoint::commit(&state, || {
        crate::window::webview_manager::prepare_content_reset(&state, true)
    })
    .is_err());
    assert_eq!(memory(&state), before);
}
