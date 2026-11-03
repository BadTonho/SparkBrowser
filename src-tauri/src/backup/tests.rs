use super::{files, merge, model::*, validation};
use crate::{
    database::{
        backup_repo,
        bookmarks_repo::{BookmarkFolder, BookmarkItem},
        connection::initialize_schema,
        saved_sessions_repo::{SavedSession, SessionPage},
    },
    state::split_layout::SplitLayout,
    updates::model::UpdateChannel,
};
use rusqlite::Connection;

fn preferences() -> BackupPreferences {
    let shortcuts = validation::SHORTCUT_IDS
        .iter()
        .enumerate()
        .map(|(i, id)| {
            let sequence = ["switch-tabs", "switch-workspaces"].contains(id);
            (
                (*id).to_owned(),
                vec![ShortcutBinding {
                    key: if sequence {
                        "Digit".into()
                    } else if *id == "workspace-picker" {
                        "Alt".into()
                    } else {
                        format!("Key{i}")
                    },
                    ctrl: *id == "switch-tabs",
                    alt: *id == "switch-workspaces",
                    shift: false,
                    meta: false,
                }],
            )
        })
        .collect();
    BackupPreferences {
        auto_skip_ads_enabled: false,
        pip_window_shadow: false,
        tab_layout: "horizontal".into(),
        show_bookmarks_bar: true,
        overlay_blur_px: 16,
        workspace_switcher_display: "toolbar".into(),
        default_search_engine: "google".into(),
        auto_hibernate_mode: serde_json::json!(10),
        resident_limit: 8,
        language: "system".into(),
        keyboard_shortcuts: shortcuts,
        site_zoom: vec![],
        update_channel: UpdateChannel::Stable,
    }
}
fn tab(id: &str, url: &str, workspace: &str, active: bool) -> BackupTab {
    BackupTab {
        id: id.into(),
        url: url.into(),
        title: format!("Title {id}"),
        workspace_id: workspace.into(),
        is_pinned: false,
        is_active: active,
    }
}
pub(super) fn setup() -> (Connection, BackupDocument) {
    let conn = Connection::open_in_memory().unwrap();
    initialize_schema(&conn).unwrap();
    let mut data = backup_repo::collect(&conn).unwrap();
    data.tabs = vec![
        tab("tab-1", "https://example.test/a", "ws-personal", true),
        tab("tab-2", "https://example.test/b", "ws-personal", false),
    ];
    crate::database::session_repo::save_session_tabs(
        &conn,
        &data.tabs.iter().map(BackupTab::tab).collect::<Vec<_>>(),
    )
    .unwrap();
    let doc = BackupDocument {
        format: FORMAT.into(),
        version: 1,
        created_at: 1,
        data,
        preferences: preferences(),
    };
    validation::document(&doc).unwrap();
    (conn, doc)
}
#[test]
fn backup_file_roundtrip_rejects_invalid_versions_sizes_and_references_without_replacing_a_file() {
    let (_conn, doc) = setup();
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("test.veloce-backup.json");
    files::write(&path, &doc).unwrap();
    let original = std::fs::read(&path).unwrap();
    let restored = files::read(&path).unwrap();
    assert_eq!(
        serde_json::to_value(&doc).unwrap(),
        serde_json::to_value(restored).unwrap()
    );
    let mut invalid = doc.clone();
    invalid.version = 5;
    assert!(files::write(&path, &invalid).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), original);
    assert!(files::decode(b"not json").is_err());
    assert!(files::decode(&vec![b' '; MAX_BYTES as usize + 1]).is_err());
    invalid = doc.clone();
    invalid.data.tabs[0].workspace_id = "missing".into();
    assert!(validation::document(&invalid).is_err());
    invalid = doc.clone();
    invalid
        .preferences
        .keyboard_shortcuts
        .insert("unknown-action".into(), vec![]);
    assert!(validation::document(&invalid).is_err());
    let mut json = serde_json::to_value(&doc).unwrap();
    json["cookies"] = serde_json::json!([]);
    assert!(files::decode(&serde_json::to_vec(&json).unwrap()).is_err());
    let blocked = directory.path().join("directory.veloce-backup.json");
    std::fs::create_dir(&blocked).unwrap();
    assert!(files::write(&blocked, &doc).is_err());
    assert_eq!(
        std::fs::read_dir(directory.path()).unwrap().count(),
        2,
        "failed atomic rename removes its temporary file"
    );
}
#[test]
fn merge_preserves_existing_selection_and_counts_duplicate_occurrences_without_reimporting() {
    let (conn, doc) = setup();
    let mut source = doc.clone();
    source.data.workspaces[0].id = "foreign-personal".into();
    source.data.workspaces[0].name = doc.data.workspaces[0].name.to_uppercase();
    source.data.workspaces[0].accent_primary = "#123456".into();
    source.data.tabs = vec![
        tab(
            "source-1",
            "https://EXAMPLE.test/a",
            "foreign-personal",
            true,
        ),
        tab(
            "source-2",
            "https://example.test/a",
            "foreign-personal",
            false,
        ),
        tab(
            "source-3",
            "https://example.test/a",
            "foreign-personal",
            false,
        ),
    ];
    let plan = merge::plan(&doc.data, &source.data, "one", 3).unwrap();
    assert_eq!(plan.added["tabs"], 2);
    assert_eq!(plan.added["workspaces"], 0);
    assert_eq!(plan.data.tabs[0].id, "tab-1");
    assert!(plan.data.tabs[0].is_active);
    assert_eq!(plan.data.tabs[1].id, "tab-2");
    assert!(plan.data.tabs[2..]
        .iter()
        .all(|t| !t.is_active && t.id.starts_with("tab-")));
    assert_eq!(
        plan.data.workspaces[0].accent_primary,
        doc.data.workspaces[0].accent_primary
    );
    backup_repo::commit(&conn, &doc.data, &plan.data, None).unwrap();
    let current = backup_repo::collect(&conn).unwrap();
    let repeated = merge::plan(&current, &source.data, "two", plan.next_tab_counter).unwrap();
    assert!(repeated.added.values().all(|n| *n == 0));
}
#[test]
fn merge_remaps_folder_ids_keeps_conflicting_values_and_uniquely_names_sessions() {
    let (conn, mut doc) = setup();
    conn.execute_batch("INSERT INTO bookmark_folders VALUES(1,'Work',1); INSERT INTO bookmarks(id,title,url,icon,created_at,folder_id) VALUES(1,'Kept','https://example.test/', 'x',1,1); INSERT INTO page_notes VALUES('https://example.test/n?q=1','Kept','Current draft',1)").unwrap();
    doc.data = backup_repo::collect(&conn).unwrap();
    let mut source = doc.clone();
    source.data.bookmarks.folders = vec![BookmarkFolder {
        id: 99,
        name: "WORK".into(),
    }];
    source.data.bookmarks.bookmarks = vec![
        BookmarkItem {
            id: 90,
            title: "Ignored".into(),
            url: "https://EXAMPLE.test".into(),
            icon: "n".into(),
            folder_id: Some(99),
        },
        BookmarkItem {
            id: 91,
            title: "New".into(),
            url: "https://example.test/new".into(),
            icon: "n".into(),
            folder_id: Some(99),
        },
    ];
    source.data.notes[0].text = "Ignored".into();
    source
        .data
        .notes
        .push(crate::database::notes_repo::PageNote {
            url: "https://example.test/n?q=2#ignored".into(),
            title: "Different query".into(),
            text: "New".into(),
            updated_at: 2,
        });
    let session = SavedSession {
        id: 1,
        name: "Session".into(),
        pages: vec![SessionPage {
            title: "A".into(),
            url: "https://example.test/a".into(),
            is_pinned: true,
            is_selected: true,
        }],
    };
    doc.data.sessions = vec![session.clone()];
    source.data.sessions = vec![session.clone(), session];
    source.data.sessions[1].pages[0].url = "https://example.test/different".into();
    let plan = merge::plan(&doc.data, &source.data, "one", 3).unwrap();
    assert_eq!(plan.data.bookmarks.bookmarks[0].title, "Kept");
    assert_eq!(plan.data.bookmarks.bookmarks[1].folder_id, Some(1));
    assert_ne!(plan.data.bookmarks.bookmarks[1].id, 91);
    assert_eq!(plan.data.notes[0].text, "Current draft");
    assert_eq!(plan.data.notes[1].url, "https://example.test/n?q=2");
    assert_eq!(plan.data.sessions.len(), 2);
    assert_eq!(plan.data.sessions[1].name, "Session (2)");
    let repeated = merge::plan(&plan.data, &source.data, "two", plan.next_tab_counter).unwrap();
    assert!(repeated.added.values().all(|n| *n == 0));
}
#[test]
fn splits_import_only_into_new_workspaces_and_preserve_all_existing_layouts() {
    let (_conn, mut doc) = setup();
    let layout = SplitLayout {
        left_tab_ids: Vec::new(),
        right_tab_ids: Vec::new(),
        workspace_id: "ws-personal".into(),
        left_tab_id: "tab-1".into(),
        right_tab_id: "tab-2".into(),
        focused_tab_id: "tab-2".into(),
        ratio: 0.4,
    };
    doc.data.split_layouts.push(layout.clone());
    let mut source = doc.clone();
    source.data.split_layouts[0].ratio = 0.7;
    let mut workspace = doc.data.workspaces[0].clone();
    workspace.id = "new".into();
    workspace.name = "Imported workspace".into();
    source.data.workspaces.push(workspace);
    source.data.tabs.extend([
        tab("left", "https://example.test/left", "new", false),
        tab("right", "https://example.test/right", "new", false),
    ]);
    source.data.split_layouts.push(SplitLayout {
        left_tab_ids: Vec::new(),
        right_tab_ids: Vec::new(),
        workspace_id: "new".into(),
        left_tab_id: "left".into(),
        right_tab_id: "right".into(),
        focused_tab_id: "right".into(),
        ratio: 0.6,
    });
    validation::document(&source).unwrap();
    let plan = merge::plan(&doc.data, &source.data, "splits", 3).unwrap();
    assert_eq!(plan.data.split_layouts[0], layout);
    let restored = &plan.data.split_layouts[1];
    assert_ne!(restored.workspace_id, "new");
    assert_ne!(restored.left_tab_id, "left");
    assert!(restored.valid(
        &plan
            .data
            .tabs
            .iter()
            .map(BackupTab::tab)
            .collect::<Vec<_>>()
    ));
}
#[test]
fn failed_import_rolls_back_data_and_outbox_while_preference_retries_never_import_again() {
    let (conn, doc) = setup();
    let mut source = doc.clone();
    source.data.bookmarks.bookmarks.push(BookmarkItem {
        id: 100,
        title: "New".into(),
        url: "https://example.test/new".into(),
        icon: "x".into(),
        folder_id: None,
    });
    let plan = merge::plan(&doc.data, &source.data, "rollback", 3).unwrap();
    let task = PreferenceTask {
        id: "task".into(),
        preferences: preferences(),
        error: None,
    };
    conn.execute_batch("CREATE TRIGGER fail_backup BEFORE INSERT ON bookmarks BEGIN SELECT RAISE(ABORT, 'Artificial storage failure'); END;").unwrap();
    assert!(backup_repo::commit(&conn, &doc.data, &plan.data, Some(&task)).is_err());
    assert!(backup_repo::pending(&conn).unwrap().is_none());
    assert_eq!(
        serde_json::to_value(backup_repo::collect(&conn).unwrap()).unwrap(),
        serde_json::to_value(&doc.data).unwrap()
    );
    conn.execute_batch("DROP TRIGGER fail_backup").unwrap();
    backup_repo::commit(&conn, &doc.data, &plan.data, Some(&task)).unwrap();
    backup_repo::acknowledge(&conn, "task", Some("Artificial preference error")).unwrap();
    assert!(backup_repo::pending(&conn)
        .unwrap()
        .unwrap()
        .error
        .is_some());
    let before = serde_json::to_vec(&backup_repo::collect(&conn).unwrap()).unwrap();
    backup_repo::acknowledge(&conn, "task", None).unwrap();
    assert!(backup_repo::pending(&conn).unwrap().is_none());
    assert_eq!(
        serde_json::to_vec(&backup_repo::collect(&conn).unwrap()).unwrap(),
        before
    );
}

#[test]
fn v1_defaults_zoom_and_v2_preserves_zoom_preferences() {
    let (_, mut doc) = setup();
    let mut v1 = serde_json::to_value(&doc).unwrap();
    v1["preferences"]
        .as_object_mut()
        .unwrap()
        .remove("site_zoom");
    assert!(files::decode(&serde_json::to_vec(&v1).unwrap())
        .unwrap()
        .preferences
        .site_zoom
        .is_empty());
    doc.version = 2;
    doc.preferences
        .site_zoom
        .push(crate::database::site_zoom_repo::SiteZoom {
            origin: "https://zoom.example".into(),
            factor: 1.5,
        });
    validation::document(&doc).unwrap();
    let roundtrip = files::decode(&serde_json::to_vec(&doc).unwrap()).unwrap();
    assert_eq!(roundtrip.preferences.site_zoom[0].factor, 1.5);
}

#[test]
fn old_backups_default_shadow_off_and_v3_roundtrips_the_boolean() {
    let (_, mut doc) = setup();
    for version in [1, 2] {
        let mut legacy = serde_json::to_value(&doc).unwrap();
        legacy["version"] = serde_json::json!(version);
        legacy["preferences"]
            .as_object_mut()
            .unwrap()
            .remove("pip_window_shadow");
        let decoded = files::decode(&serde_json::to_vec(&legacy).unwrap()).unwrap();
        assert!(!decoded.preferences.pip_window_shadow);
    }
    doc.version = 3;
    doc.preferences.pip_window_shadow = true;
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("v3.veloce-backup.json");
    files::write(&path, &doc).unwrap();
    let decoded = files::read(&path).unwrap();
    assert!(decoded.preferences.pip_window_shadow);
    let mut invalid = serde_json::to_value(&doc).unwrap();
    invalid["preferences"]["pip_window_shadow"] = serde_json::json!("true");
    assert!(files::decode(&serde_json::to_vec(&invalid).unwrap()).is_err());
}

#[test]
fn v4_groups_roundtrip_and_reject_duplicate_or_missing_membership() {
    let (_, mut doc) = setup();
    let mut tab = doc.data.tabs[0].clone();
    tab.id = "group-b".into();
    tab.is_active = false;
    doc.data.tabs.push(tab);
    let mut layout = SplitLayout {
        workspace_id: doc.data.tabs[0].workspace_id.clone(),
        left_tab_id: doc.data.tabs[0].id.clone(),
        right_tab_id: "group-b".into(),
        focused_tab_id: "group-b".into(),
        ratio: 0.6,
        left_tab_ids: Vec::new(),
        right_tab_ids: Vec::new(),
    };
    layout.normalize(&doc.data.tabs.iter().map(BackupTab::tab).collect::<Vec<_>>());
    doc.version = 4;
    doc.data.split_layouts.push(layout);
    validation::document(&doc).unwrap();
    let restored = files::decode(&serde_json::to_vec(&doc).unwrap()).unwrap();
    assert_eq!(restored.data.split_layouts, doc.data.split_layouts);
    doc.data.split_layouts[0]
        .left_tab_ids
        .push("group-b".into());
    assert!(validation::document(&doc).is_err());
    doc.data.split_layouts[0].left_tab_ids.clear();
    assert!(validation::document(&doc).is_err());
}

#[test]
fn auto_skip_backup_defaults_off_and_accepts_only_booleans() {
    let (_, mut doc) = setup();
    doc.preferences.auto_skip_ads_enabled = true;
    let mut value = serde_json::to_value(&doc).unwrap();
    let decoded: BackupDocument = serde_json::from_value(value.clone()).unwrap();
    assert!(decoded.preferences.auto_skip_ads_enabled);
    value["preferences"]
        .as_object_mut()
        .unwrap()
        .remove("auto_skip_ads_enabled");
    let decoded: BackupDocument = serde_json::from_value(value.clone()).unwrap();
    assert!(!decoded.preferences.auto_skip_ads_enabled);
    value["preferences"]["auto_skip_ads_enabled"] = serde_json::json!("true");
    assert!(serde_json::from_value::<BackupDocument>(value).is_err());
}
