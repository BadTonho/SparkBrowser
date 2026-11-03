use super::SplitLayout;
use crate::{
    database::{connection::initialize_schema, split_repo},
    state::{
        browser_state::BrowserState,
        split_docking::{self, DockContext, DockSide},
        split_transfer,
        tab_model::TabInfo,
    },
};
use rusqlite::Connection;

fn fixture() -> BrowserState {
    let state = BrowserState::default();
    let db = Connection::open_in_memory().unwrap();
    initialize_schema(&db).unwrap();
    *state.db.lock().unwrap() = Some(db);
    for id in ["b", "c", "d"] {
        state.tabs.lock().unwrap().push(TabInfo::new_default(
            id.into(),
            "https://example.test/".into(),
            false,
        ));
    }
    state.persist_session().unwrap();
    split_docking::commit(&state, &context(&state), "b", DockSide::Right).unwrap();
    state
}
fn context(state: &BrowserState) -> DockContext {
    let split = state.current_split();
    DockContext {
        workspace_id: "ws-personal".into(),
        active_tab_id: state.active_tab_id.lock().unwrap().clone(),
        split_key: split.as_ref().map(SplitLayout::key),
        groups_key: split.as_ref().map(SplitLayout::groups_key),
    }
}

#[test]
fn groups_select_independently_and_only_visible_pages_are_protected() {
    let state = fixture();
    let first = state.current_split().unwrap();
    assert_eq!(first.left_tab_ids, ["tab-1", "c", "d"]);
    assert_eq!(first.right_tab_ids, ["b"]);
    state.select_tab("c").unwrap();
    state.refresh_visible_tabs().unwrap();
    assert_eq!(state.current_split().unwrap().left_tab_id, "c");
    state.select_tab("b").unwrap();
    assert_eq!(state.current_split().unwrap().left_tab_id, "c");
    let tabs = state.snapshot_tabs();
    assert_eq!(
        tabs.iter()
            .filter(|t| t.is_visible)
            .map(|t| t.id.as_str())
            .collect::<Vec<_>>(),
        ["b", "c"]
    );
    assert!(!tabs
        .iter()
        .find(|t| t.id == "d")
        .unwrap()
        .protected_from_hibernation());
    state.persist_session().unwrap();
    assert_eq!(
        split_repo::load(state.db.lock().unwrap().as_ref().unwrap()).unwrap()["ws-personal"],
        state.current_split().unwrap()
    );
}

#[test]
fn transfer_reorders_group_slots_preserves_selection_and_rejects_stale_context() {
    let state = fixture();
    let original_context = context(&state);
    split_transfer::commit(&state, &original_context, "c", DockSide::Right, Some("b")).unwrap();
    let split = state.current_split().unwrap();
    assert_eq!(split.right_tab_ids, ["c", "b"]);
    assert_eq!(split.left_tab_id, "tab-1");
    assert_eq!(split.right_tab_id, "c");
    assert!(split_transfer::commit(&state, &original_context, "d", DockSide::Right, None).is_err());
    split_transfer::commit(&state, &context(&state), "c", DockSide::Right, None).unwrap();
    assert_eq!(state.current_split().unwrap().right_tab_ids, ["b", "c"]);
    assert_eq!(state.current_split().unwrap().right_tab_id, "c");
    assert_eq!(
        state
            .snapshot_tabs()
            .iter()
            .map(|t| t.id.as_str())
            .collect::<Vec<_>>(),
        ["tab-1", "b", "c", "d"]
    );
}

#[test]
fn transfer_failure_rolls_back_group_order_selection_and_database() {
    let state = fixture();
    let before = serde_json::to_value(state.snapshot_packet()).unwrap();
    state.db.lock().unwrap().as_ref().unwrap().execute_batch("CREATE TRIGGER fail_groups BEFORE INSERT ON workspace_splits BEGIN SELECT RAISE(ABORT,'group write failure'); END;").unwrap();
    assert!(
        split_transfer::commit(&state, &context(&state), "c", DockSide::Right, Some("b")).is_err()
    );
    let after = state.snapshot_packet();
    assert_eq!(serde_json::to_value(&after.tabs).unwrap(), before["tabs"]);
    assert_eq!(
        serde_json::to_value(&after.split_layout).unwrap(),
        before["split_layout"]
    );
    state
        .db
        .lock()
        .unwrap()
        .as_ref()
        .unwrap()
        .execute_batch("DROP TRIGGER fail_groups")
        .unwrap();
    split_transfer::commit(&state, &context(&state), "c", DockSide::Right, None).unwrap();
}

#[test]
fn removal_chooses_right_neighbor_then_left_and_empty_group_ends_split() {
    let state = fixture();
    state.select_tab("c").unwrap();
    let mut split = state.current_split().unwrap();
    assert!(split.remove_member("c"));
    assert_eq!(split.left_tab_id, "d");
    assert_eq!(split.focused_tab_id, "d");
    assert!(split.remove_member("d"));
    assert_eq!(split.left_tab_id, "tab-1");
    assert!(!split.remove_member("tab-1"));
    split_transfer::commit(&state, &context(&state), "b", DockSide::Left, None).unwrap();
    assert!(state.current_split().is_none());
    assert_eq!(*state.active_tab_id.lock().unwrap(), "b");
}

#[test]
fn workspace_transfer_repairs_source_and_assigns_destination_focused_group() {
    let state = fixture();
    state.move_tab("c", "ws-work").unwrap();
    state.refresh_visible_tabs().unwrap();
    assert_eq!(state.current_split().unwrap().left_tab_ids, ["tab-1", "d"]);
    state.move_tab("b", "ws-work").unwrap();
    state.refresh_visible_tabs().unwrap();
    assert!(!state
        .split_layouts
        .lock()
        .unwrap()
        .contains_key("ws-personal"));
}
