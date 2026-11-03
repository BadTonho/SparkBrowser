use super::*;
use crate::{
    database::{connection::initialize_schema, split_repo},
    state::tab_checkpoint::TabCheckpoint,
};
use rusqlite::Connection;

fn state() -> BrowserState {
    let state = BrowserState::default();
    let conn = Connection::open_in_memory().unwrap();
    initialize_schema(&conn).unwrap();
    *state.db.lock().unwrap() = Some(conn);
    state.tabs.lock().unwrap().push(TabInfo::new_default(
        "b".into(),
        "https://example.test/b".into(),
        false,
    ));
    state.split_layouts.lock().unwrap().insert(
        "ws-personal".into(),
        SplitLayout {
            left_tab_ids: Vec::new(),
            right_tab_ids: Vec::new(),
            workspace_id: "ws-personal".into(),
            left_tab_id: "tab-1".into(),
            right_tab_id: "b".into(),
            focused_tab_id: "tab-1".into(),
            ratio: 0.5,
        },
    );
    state.persist_session().unwrap();
    state
}

#[test]
fn live_previews_never_enter_session_saves_and_cancel_restores_geometry() {
    let state = state();
    let key = state.current_split().unwrap().key();
    state.preview_split_ratio(&key, 0.7).unwrap();
    assert_eq!(state.display_split().unwrap().ratio, 0.7);
    assert_eq!(state.snapshot_packet().split_layout.unwrap().ratio, 0.5);
    state.persist_session().unwrap();
    let db = state.db.lock().unwrap();
    assert_eq!(
        split_repo::load(db.as_ref().unwrap()).unwrap()["ws-personal"].ratio,
        0.5
    );
    drop(db);
    state.preview_split_ratio(&key, 0.5).unwrap();
    assert_eq!(state.display_split().unwrap().ratio, 0.5);
    assert!(state.preview_split_ratio("stale", 0.7).is_err());
    assert!(state.preview_split_ratio(&key, f64::NAN).is_err());
}

#[test]
fn failed_commit_discards_preview_without_changing_the_saved_ratio() {
    let state = state();
    let key = state.current_split().unwrap().key();
    state.preview_split_ratio(&key, 0.7).unwrap();
    state.db.lock().unwrap().as_ref().unwrap().execute_batch(
        "CREATE TRIGGER fail_ratio BEFORE UPDATE ON workspace_splits BEGIN SELECT RAISE(ABORT,'injected ratio failure'); END;"
    ).unwrap();
    assert!(
        TabCheckpoint::commit(&state, || crate::commands::split_view::update_ratio(
            &state, &key, 0.7
        ))
        .is_err()
    );
    state.clear_split_preview(&key).unwrap();
    assert_eq!(state.current_split().unwrap().ratio, 0.5);
    assert_eq!(state.display_split().unwrap().ratio, 0.5);
    let db = state.db.lock().unwrap();
    assert_eq!(
        split_repo::load(db.as_ref().unwrap()).unwrap()["ws-personal"].ratio,
        0.5
    );
}
