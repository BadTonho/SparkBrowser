//! Selective restore (groups) and replace mode.
use super::{merge, model::*, tests::setup};
use crate::database::{backup_repo, bookmarks_repo::BookmarkItem, notes_repo::PageNote};

fn options(groups: &[RestoreGroup], mode: RestoreMode) -> RestoreOptions {
    RestoreOptions {
        groups: groups.to_vec(),
        mode,
    }
}
fn bookmark(id: i64, url: &str) -> BookmarkItem {
    BookmarkItem {
        id,
        title: format!("Bookmark {id}"),
        url: url.into(),
        icon: "x".into(),
        folder_id: None,
    }
}
fn note(url: &str) -> PageNote {
    PageNote {
        url: url.into(),
        title: "Note".into(),
        text: "Backup text".into(),
        updated_at: 1,
    }
}

#[test]
fn unselected_groups_are_left_out_of_the_plan() {
    let (_conn, doc) = setup();
    let mut source = doc.clone();
    source.data.bookmarks.bookmarks = vec![bookmark(7, "https://example.test/b1")];
    source.data.notes = vec![note("https://example.test/n1")];
    let only_notes = options(&[RestoreGroup::Notes], RestoreMode::Merge);
    let plan = merge::plan_with(&doc.data, &source.data, "sel", 3, &only_notes).unwrap();
    assert_eq!(plan.added["notes"], 1);
    assert_eq!(plan.added["bookmarks"], 0);
    assert_eq!(plan.added["tabs"], 0);
    assert!(plan.replaced.is_empty() && plan.removed.is_empty());
}

#[test]
fn workspaces_group_carries_its_tabs() {
    let (_conn, doc) = setup();
    let mut source = doc.clone();
    source.data.workspaces[0].id = "foreign".into();
    source.data.workspaces[0].name = "Foreign".into();
    for tab in &mut source.data.tabs {
        tab.workspace_id = "foreign".into();
        tab.url.push_str("?foreign");
    }
    let without = options(&[RestoreGroup::Bookmarks], RestoreMode::Merge);
    let plan = merge::plan_with(&doc.data, &source.data, "ws", 3, &without).unwrap();
    assert_eq!(plan.added["workspaces"] + plan.added["tabs"], 0);
    let with = options(&[RestoreGroup::Workspaces], RestoreMode::Merge);
    let plan = merge::plan_with(&doc.data, &source.data, "ws", 3, &with).unwrap();
    assert_eq!(plan.added["workspaces"], 1);
    assert_eq!(plan.added["tabs"], 2);
}

#[test]
fn replace_swaps_selected_groups_only_and_commits_atomically() {
    let (conn, mut doc) = setup();
    conn.execute_batch(
        "INSERT INTO bookmarks(id,title,url,icon,created_at) VALUES(1,'Old','https://example.test/old','x',1);
         INSERT INTO page_notes VALUES('https://example.test/keep','Keep','Local note',1)",
    )
    .unwrap();
    doc.data = backup_repo::collect(&conn).unwrap();
    let mut source = doc.clone();
    source.data.bookmarks.bookmarks = vec![bookmark(5, "https://example.test/new")];
    source.data.notes = vec![note("https://example.test/incoming")];
    let replace = options(&[RestoreGroup::Bookmarks], RestoreMode::Replace);
    let plan = merge::plan_with(&doc.data, &source.data, "rep", 3, &replace).unwrap();
    assert_eq!(plan.removed["bookmarks"], 1);
    assert_eq!(plan.added["bookmarks"], 1);
    backup_repo::commit_with(&conn, &doc.data, &plan.data, None, &plan.replaced).unwrap();
    let after = backup_repo::collect(&conn).unwrap();
    let urls: Vec<_> = after.bookmarks.bookmarks.iter().map(|b| &b.url).collect();
    assert_eq!(urls, ["https://example.test/new"]);
    // Notes were not selected, so the local note survives and the incoming one stays out.
    assert_eq!(after.notes.len(), 1);
    assert_eq!(after.notes[0].url, "https://example.test/keep");
}

#[test]
fn replace_never_touches_workspaces_even_when_selected() {
    let (_conn, doc) = setup();
    let all = options(&RestoreGroup::ALL, RestoreMode::Replace);
    let plan = merge::plan_with(&doc.data, &doc.data, "all", 3, &all).unwrap();
    assert!(!plan.replaced.contains(&RestoreGroup::Workspaces));
    assert_eq!(plan.data.workspaces.len(), doc.data.workspaces.len());
}

#[test]
fn empty_or_duplicated_selection_is_rejected() {
    let (_conn, doc) = setup();
    for groups in [vec![], vec![RestoreGroup::Notes, RestoreGroup::Notes]] {
        let bad = RestoreOptions {
            groups,
            mode: RestoreMode::Merge,
        };
        assert!(merge::plan_with(&doc.data, &doc.data, "bad", 3, &bad).is_err());
    }
}

#[test]
fn preference_groups_mix_backup_and_current_values() {
    let (_conn, doc) = setup();
    let current = doc.preferences.clone();
    let mut incoming = current.clone();
    incoming.tab_layout = "vertical".into();
    incoming.keyboard_shortcuts.clear();
    let pick = |groups: &[RestoreGroup]| options(groups, RestoreMode::Merge);
    assert!(pick(&[RestoreGroup::Notes])
        .preferences(&current, &incoming)
        .is_none());
    let settings = pick(&[RestoreGroup::Settings])
        .preferences(&current, &incoming)
        .unwrap();
    assert_eq!(settings.tab_layout, "vertical");
    assert_eq!(
        settings.keyboard_shortcuts.len(),
        current.keyboard_shortcuts.len()
    );
    let shortcuts = pick(&[RestoreGroup::Shortcuts])
        .preferences(&current, &incoming)
        .unwrap();
    assert_eq!(shortcuts.tab_layout, current.tab_layout);
    assert!(shortcuts.keyboard_shortcuts.is_empty());
}
