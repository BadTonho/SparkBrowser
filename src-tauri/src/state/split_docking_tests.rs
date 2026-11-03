use super::*;
use crate::database::{connection::initialize_schema, session_repo, split_repo};
use rusqlite::Connection;

fn fixture() -> (BrowserState, DockContext) {
    let state = BrowserState::default();
    let db = Connection::open_in_memory().unwrap();
    initialize_schema(&db).unwrap();
    *state.db.lock().unwrap() = Some(db);
    for id in ["tab-2", "tab-3"] {
        state.tabs.lock().unwrap().push(TabInfo::new_default(
            id.into(),
            "https://example.test/".into(),
            false,
        ));
    }
    for (i, tab) in state.tabs.lock().unwrap().iter_mut().enumerate() {
        tab.last_active_epoch = i as u64;
    }
    state.persist_session().unwrap();
    (
        state,
        DockContext {
            groups_key: None,
            workspace_id: "ws-personal".into(),
            active_tab_id: "tab-1".into(),
            split_key: None,
        },
    )
}
#[test]
fn new_split_uses_active_or_most_recent_other_and_original_order_for_ties() {
    let (state, context) = fixture();
    let next = plan(&state, &context, "tab-2", DockSide::Left).unwrap();
    assert_eq!(
        (&*next.left_tab_id, &*next.right_tab_id, next.ratio),
        ("tab-2", "tab-1", 0.5)
    );
    let next = plan(&state, &context, "tab-1", DockSide::Right).unwrap();
    assert_eq!(next.left_tab_id, "tab-3");
    state.tabs.lock().unwrap()[1].last_active_epoch = 2;
    assert_eq!(
        plan(&state, &context, "tab-1", DockSide::Right)
            .unwrap()
            .left_tab_id,
        "tab-2"
    );
    state.tabs.lock().unwrap().truncate(1);
    assert!(plan(&state, &context, "tab-1", DockSide::Left).is_err());
}
#[test]
fn replacement_swaps_existing_members_and_preserves_ratio_order_and_selection() {
    let (state, mut context) = fixture();
    commit(&state, &context, "tab-2", DockSide::Right).unwrap();
    let mut split = state.current_split().unwrap();
    split.ratio = 0.65;
    state
        .split_layouts
        .lock()
        .unwrap()
        .insert(context.workspace_id.clone(), split.clone());
    context.active_tab_id = "tab-2".into();
    context.split_key = Some(split.key());
    let swap = plan(&state, &context, "tab-2", DockSide::Left).unwrap();
    assert_eq!(
        (&*swap.left_tab_id, &*swap.right_tab_id, swap.ratio),
        ("tab-2", "tab-1", 0.65)
    );
    let same = plan(&state, &context, "tab-2", DockSide::Right).unwrap();
    assert_eq!(same, split);
    commit(&state, &context, "tab-3", DockSide::Left).unwrap();
    let packet = state.snapshot_packet();
    assert_eq!(
        packet
            .tabs
            .iter()
            .map(|t| t.id.as_str())
            .collect::<Vec<_>>(),
        ["tab-1", "tab-2", "tab-3"]
    );
    assert_eq!(
        packet.tabs.iter().find(|t| t.is_active).unwrap().id,
        "tab-3"
    );
    let split = packet.split_layout.unwrap();
    assert_eq!(split.right_tab_id, "tab-2");
    assert_eq!(split.ratio, 0.65);
}
#[test]
fn changed_workspace_active_split_or_dragged_tab_invalidates_drop() {
    let (state, mut context) = fixture();
    context.workspace_id = "ws-work".into();
    assert!(plan(&state, &context, "tab-2", DockSide::Left).is_err());
    context.workspace_id = "ws-personal".into();
    context.active_tab_id = "tab-3".into();
    assert!(plan(&state, &context, "tab-2", DockSide::Left).is_err());
    context.active_tab_id = "tab-1".into();
    context.split_key = Some("stale".into());
    assert!(plan(&state, &context, "tab-2", DockSide::Left).is_err());
    context.split_key = None;
    state.tabs.lock().unwrap()[1].workspace_id = "ws-work".into();
    assert!(plan(&state, &context, "tab-2", DockSide::Left).is_err());
}
#[test]
fn activation_failure_rolls_back_session_split_and_residency_before_retry() {
    let (state, context) = fixture();
    let before = serde_json::to_value(state.snapshot_tabs()).unwrap();
    state.db.lock().unwrap().as_ref().unwrap().execute_batch("CREATE TRIGGER fail_dock BEFORE INSERT ON tab_usage_events BEGIN SELECT RAISE(ABORT,'dock activation failure'); END;").unwrap();
    assert!(commit(&state, &context, "tab-2", DockSide::Left)
        .unwrap_err()
        .contains("dock activation failure"));
    assert_eq!(serde_json::to_value(state.snapshot_tabs()).unwrap(), before);
    assert!(state.current_split().is_none());
    let guard = state.db.lock().unwrap();
    let db = guard.as_ref().unwrap();
    assert!(split_repo::load(db).unwrap().is_empty());
    assert_eq!(
        session_repo::load_session_tabs(db)
            .unwrap()
            .iter()
            .find(|t| t.is_active)
            .unwrap()
            .id,
        "tab-1"
    );
    db.execute_batch("DROP TRIGGER fail_dock;").unwrap();
    drop(guard);
    commit(&state, &context, "tab-2", DockSide::Left).unwrap();
    assert_eq!(state.current_split().unwrap().left_tab_id, "tab-2");
}
