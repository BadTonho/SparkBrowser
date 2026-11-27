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
        workspace_isolation: false,
        settings_display: "popup".into(),
        tab_layout: "horizontal".into(),
        show_bookmarks_bar: true,
        overlay_blur_px: 16,
        workspace_switcher_display: "toolbar".into(),
        default_search_engine: "google".into(),
        auto_hibernate_mode: serde_json::json!(10),
        resident_limit: 8,
        hibernation_exemptions: vec![],
        ask_download_destination: true,
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
        device: None,
        data,
        preferences: preferences(),
    };
    validation::document(&doc).unwrap();
    (conn, doc)
}
#[test]
fn new_tab_roundtrips_in_v4_tabs_and_sessions_without_relaxing_other_urls() {
    let (_db, mut doc) = setup();
    doc.version = 4;
    doc.data.tabs[0].url = crate::window::new_tab::URL.into();
    doc.data.sessions.push(SavedSession {
        id: 99,
        name: "Local page".into(),
        pages: vec![SessionPage {
            title: "New tab".into(),
            url: crate::window::new_tab::URL.into(),
            is_pinned: false,
            is_selected: true,
        }],
    });
    validation::document(&doc).unwrap();
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("local.spark-backup.json");
    files::write(&path, &doc).unwrap();
    let restored = files::read(&path).unwrap();
    assert_eq!(restored.data.tabs[0].url, crate::window::new_tab::URL);
    let mut current = doc.data.clone();
    current.tabs.clear();
    current.sessions.clear();
    let merged = merge::plan(&current, &restored.data, "local-test", 10).unwrap();
    assert_eq!(merged.data.tabs[0].url, crate::window::new_tab::URL);
    assert_eq!(
        merged.data.sessions[0].pages[0].url,
        crate::window::new_tab::URL
    );
    doc.data.bookmarks.bookmarks.push(BookmarkItem {
        id: 99,
        title: "Invalid".into(),
        url: crate::window::new_tab::URL.into(),
        icon: "".into(),
        folder_id: None,
    });
    assert!(validation::document(&doc).is_err());
}
#[test]
fn backup_file_roundtrip_rejects_invalid_versions_sizes_and_references_without_replacing_a_file() {
    let (_conn, doc) = setup();
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("test.spark-backup.json");
    files::write(&path, &doc).unwrap();
    let original = std::fs::read(&path).unwrap();
    let restored = files::read(&path).unwrap();
    assert_eq!(
        serde_json::to_value(&doc).unwrap(),
        serde_json::to_value(restored).unwrap()
    );
    let mut invalid = doc.clone();
    invalid.version = CURRENT_VERSION + 1;
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
    let blocked = directory.path().join("directory.spark-backup.json");
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
fn old_backups_default_new_settings_and_v3_roundtrips_them() {
    let (_, mut doc) = setup();
    for version in [1, 2] {
        let mut legacy = serde_json::to_value(&doc).unwrap();
        legacy["version"] = serde_json::json!(version);
        legacy["preferences"]
            .as_object_mut()
            .unwrap()
            .remove("pip_window_shadow");
        legacy["preferences"]
            .as_object_mut()
            .unwrap()
            .remove("hibernation_exemptions");
        legacy["preferences"]
            .as_object_mut()
            .unwrap()
            .remove("ask_download_destination");
        let decoded = files::decode(&serde_json::to_vec(&legacy).unwrap()).unwrap();
        assert!(!decoded.preferences.pip_window_shadow);
        assert!(decoded.preferences.hibernation_exemptions.is_empty());
        assert!(decoded.preferences.ask_download_destination);
    }
    doc.version = 3;
    doc.preferences.pip_window_shadow = true;
    doc.preferences.hibernation_exemptions = vec!["example.com".into()];
    doc.preferences.ask_download_destination = false;
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("v3.spark-backup.json");
    files::write(&path, &doc).unwrap();
    let decoded = files::read(&path).unwrap();
    assert!(decoded.preferences.pip_window_shadow);
    assert_eq!(decoded.preferences.hibernation_exemptions, ["example.com"]);
    assert!(!decoded.preferences.ask_download_destination);
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

#[test]
fn folder_sync_pushes_skips_unchanged_and_pulls_without_leaking_sync_paths() {
    let (conn, mut doc) = setup();
    crate::database::preferences_repo::initialize_preferences_table(&conn).unwrap();
    let runtime = super::sync::SyncRuntime::default();
    let dir = tempfile::tempdir().unwrap();
    let dir_str = dir.path().to_string_lossy().into_owned();

    let state = super::sync::save_config(
        &conn,
        &runtime,
        super::sync::SyncTargetConfig::Folder {
            directory_path: dir_str.clone(),
            auto_save: true,
            apply_preferences_on_direct_pull: true,
        },
        None,
    )
    .unwrap();
    assert!(matches!(
        state.config,
        super::sync::SyncTargetConfig::Folder { .. }
    ));

    // First push writes the file
    let first = super::sync::push_document_to_target(&conn, &runtime, &doc, true).unwrap();
    assert!(first.synced);
    assert!(!first.skipped_unchanged);
    assert!(dir.path().join(super::sync::SYNC_FILE_NAME).is_file());

    // Second push with identical data/preferences and only_if_changed=true skips I/O
    let second = super::sync::push_document_to_target(&conn, &runtime, &doc, true).unwrap();
    assert!(!second.synced);
    assert!(second.skipped_unchanged);

    // Changing a preference triggers a write
    doc.preferences.resident_limit += 1;
    let third = super::sync::push_document_to_target(&conn, &runtime, &doc, true).unwrap();
    assert!(third.synced);
    assert!(!third.skipped_unchanged);

    // Pulling reads the updated document and never leaks sync config inside the backup file
    let pulled = super::sync::pull_document_from_target(&conn, &runtime).unwrap();
    assert_eq!(
        pulled.preferences.resident_limit,
        doc.preferences.resident_limit
    );
    let raw_file = std::fs::read_to_string(dir.path().join(super::sync::SYNC_FILE_NAME)).unwrap();
    assert!(!raw_file.contains(&dir_str));
}

#[test]
fn folder_sync_encrypts_when_enabled_and_pull_uses_the_stored_password() {
    let (conn, doc) = setup();
    crate::database::preferences_repo::initialize_preferences_table(&conn).unwrap();
    let runtime = super::sync::SyncRuntime::default();
    let dir = tempfile::tempdir().unwrap();
    super::sync::save_config(
        &conn,
        &runtime,
        super::sync::SyncTargetConfig::Folder {
            directory_path: dir.path().to_string_lossy().into_owned(),
            auto_save: true,
            apply_preferences_on_direct_pull: false,
        },
        None,
    )
    .unwrap();
    super::encryption::configure(
        &conn,
        &runtime.credentials,
        true,
        Some("correct horse".into()),
    )
    .unwrap();
    let pushed = super::sync::push_document_to_target(&conn, &runtime, &doc, false).unwrap();
    assert!(pushed.synced);
    let path = dir.path().join(super::sync::SYNC_FILE_NAME);
    let raw = std::fs::read_to_string(&path).unwrap();
    assert!(raw.contains("spark-backup-encrypted") && !raw.contains(&doc.data.tabs[0].url));
    let pulled = super::sync::pull_document_from_target(&conn, &runtime).unwrap();
    assert_eq!(pulled.data.tabs[0].url, doc.data.tabs[0].url);

    // Without the stored password the pull reports it instead of leaking or crashing.
    super::encryption::configure(&conn, &runtime.credentials, false, None).unwrap();
    assert_eq!(
        super::sync::pull_document_from_target(&conn, &runtime)
            .err()
            .unwrap(),
        "backup.password_required"
    );
}

#[test]
fn direct_webdav_validates_urls_respects_remember_secret_and_roundtrips_over_http() {
    use crate::sync::Mutex;
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::sync::Arc;

    let (conn, doc) = setup();
    crate::database::preferences_repo::initialize_preferences_table(&conn).unwrap();
    let runtime = super::sync::SyncRuntime::default();

    // Remote plain HTTP must be rejected; HTTPS and loopback HTTP are allowed
    assert!(super::sync::validate_webdav_url("http://cloud.example.com/dav/").is_err());
    assert!(super::sync::validate_webdav_url("https://user:pass@cloud.example.com/dav/").is_err());
    let resolved =
        super::sync::resolve_webdav_file_url("https://cloud.example.com/dav/files/u").unwrap();
    assert_eq!(
        resolved.as_str(),
        "https://cloud.example.com/dav/files/u/browser.spark-backup.json"
    );

    // Session-only vs injected secure storage; never use the real OS keyring.
    super::sync::save_config(
        &conn,
        &runtime,
        super::sync::SyncTargetConfig::DirectWebdav {
            endpoint_url: "https://cloud.example.com/dav".into(),
            username: "alice".into(),
            remember_secret: false,
            auto_save: true,
            apply_preferences_on_direct_pull: false,
        },
        Some("session-only-secret".into()),
    )
    .unwrap();
    assert!(crate::database::preferences_repo::load(
        &conn,
        crate::database::preferences_repo::BACKUP_SYNC_SECRET_KEY
    )
    .unwrap()
    .is_none());
    assert_eq!(
        super::sync::resolve_secret(&conn, &runtime)
            .unwrap()
            .as_deref(),
        Some("session-only-secret")
    );

    // Remembered credentials survive a runtime restart without entering SQLite.
    super::sync::save_config(
        &conn,
        &runtime,
        super::sync::SyncTargetConfig::DirectWebdav {
            endpoint_url: "https://cloud.example.com/dav".into(),
            username: "alice".into(),
            remember_secret: true,
            auto_save: true,
            apply_preferences_on_direct_pull: false,
        },
        None,
    )
    .unwrap();
    assert!(crate::database::preferences_repo::load(
        &conn,
        crate::database::preferences_repo::BACKUP_SYNC_SECRET_KEY
    )
    .unwrap()
    .is_none());
    let reopened = super::sync::SyncRuntime {
        credentials: runtime.credentials.clone(),
        ..Default::default()
    };
    assert_eq!(
        super::sync::resolve_secret(&conn, &reopened)
            .unwrap()
            .as_deref(),
        Some("session-only-secret")
    );

    // Local WebDAV test server for PUT + GET roundtrip
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let stored_body: Arc<Mutex<Vec<u8>>> = Arc::new(Mutex::new(Vec::new()));
    let stored_clone = stored_body.clone();

    let server = std::thread::spawn(move || {
        for _ in 0..2 {
            let (mut stream, _) = listener.accept().unwrap();
            let mut buf = Vec::new();
            let mut header_end = None;
            let mut temp = [0u8; 4096];
            while header_end.is_none() {
                let n = stream.read(&mut temp).unwrap();
                if n == 0 {
                    break;
                }
                buf.extend_from_slice(&temp[..n]);
                header_end = buf.windows(4).position(|w| w == b"\r\n\r\n").map(|p| p + 4);
            }
            let header_end = header_end.unwrap();
            let headers = String::from_utf8_lossy(&buf[..header_end]).to_string();
            assert!(headers.to_lowercase().contains("authorization: basic "));
            if headers.starts_with("PUT ") {
                let content_len = headers
                    .lines()
                    .find_map(|line| {
                        let lower = line.to_lowercase();
                        lower
                            .strip_prefix("content-length:")
                            .and_then(|v| v.trim().parse::<usize>().ok())
                    })
                    .unwrap_or(0);
                while buf.len() < header_end + content_len {
                    let n = stream.read(&mut temp).unwrap();
                    if n == 0 {
                        break;
                    }
                    buf.extend_from_slice(&temp[..n]);
                }
                *stored_clone.lock().unwrap() = buf[header_end..header_end + content_len].to_vec();
                stream
                    .write_all(
                        b"HTTP/1.1 201 Created\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
                    )
                    .unwrap();
            } else if headers.starts_with("GET ") {
                let payload = stored_clone.lock().unwrap().clone();
                let resp = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                    payload.len()
                );
                stream.write_all(resp.as_bytes()).unwrap();
                stream.write_all(&payload).unwrap();
            }
        }
    });

    super::sync::save_config(
        &conn,
        &runtime,
        super::sync::SyncTargetConfig::DirectWebdav {
            endpoint_url: format!("http://{addr}/dav"),
            username: "alice".into(),
            remember_secret: false,
            auto_save: true,
            apply_preferences_on_direct_pull: true,
        },
        Some("secret-token".into()),
    )
    .unwrap();

    let push_res = super::sync::push_document_to_target(&conn, &runtime, &doc, true).unwrap();
    assert!(push_res.synced);

    // Unchanged push is skipped without hitting network again
    let skipped = super::sync::push_document_to_target(&conn, &runtime, &doc, true).unwrap();
    assert!(skipped.skipped_unchanged);

    let pulled = super::sync::pull_document_from_target(&conn, &runtime).unwrap();
    assert_eq!(pulled.data.tabs.len(), doc.data.tabs.len());
    server.join().unwrap();
}

#[test]
fn google_drive_web_mode_persists_and_restricts_allowed_webview_hosts() {
    let (conn, _doc) = setup();
    crate::database::preferences_repo::initialize_preferences_table(&conn).unwrap();
    let runtime = super::sync::SyncRuntime::default();

    let state = super::sync::save_config(
        &conn,
        &runtime,
        super::sync::SyncTargetConfig::GoogleDriveWeb {
            auto_save: true,
            apply_preferences_on_direct_pull: true,
        },
        None,
    )
    .unwrap();
    assert_eq!(
        state.config,
        super::sync::SyncTargetConfig::GoogleDriveWeb {
            auto_save: true,
            apply_preferences_on_direct_pull: true,
        }
    );

    assert!(super::gdrive_webview::is_allowed_gdrive_host(
        &url::Url::parse("https://drive.google.com/drive/my-drive").unwrap()
    ));
    assert!(super::gdrive_webview::is_allowed_gdrive_host(
        &url::Url::parse("https://clients6.google.com/drive/v3/files").unwrap()
    ));
    assert!(!super::gdrive_webview::is_allowed_gdrive_host(
        &url::Url::parse("http://drive.google.com/drive/my-drive").unwrap()
    ));
    assert!(!super::gdrive_webview::is_allowed_gdrive_host(
        &url::Url::parse("https://evil.example.com/drive.google.com").unwrap()
    ));

    let gdrive_rt = super::gdrive_webview::GDriveWebRuntime::default();
    let token = crate::window::webview_manager::create_internal_bridge_token();
    *gdrive_rt.bridge_token.lock().unwrap() = Some(token.clone());

    super::gdrive_webview::apply_bridge_navigation(
        &gdrive_rt,
        &url::Url::parse(&format!(
            "spark-internal://google-drive-sync/status?token={token}&reason=ready&data=user%40gmail.com"
        ))
        .unwrap(),
    );
    assert_eq!(
        gdrive_rt.page_status.lock().unwrap().as_deref(),
        Some("ready")
    );
    assert_eq!(
        gdrive_rt.connected_email.lock().unwrap().as_deref(),
        Some("user@gmail.com")
    );
    assert_eq!(
        super::gdrive_webview::extract_authuser_from_url(
            "https://drive.google.com/drive/u/2/my-drive"
        ),
        Some("2".to_string())
    );
    assert_eq!(
        super::gdrive_webview::extract_authuser_from_url("https://drive.google.com/drive/my-drive"),
        Some("0".to_string())
    );
    assert_eq!(
        super::gdrive_webview::sha1_hex(b"abc"),
        "a9993e364706816aba3e25717850c26c9cd0d89d"
    );
    let sample_html = r#"["https://clients6.google.com/drive/v2internal/","AIzaSyD12345678901234567890123456789012"]"#;
    let extracted_keys = super::gdrive_webview::extract_api_keys_from_html(sample_html);
    assert_eq!(
        extracted_keys,
        vec!["AIzaSyD12345678901234567890123456789012".to_string()]
    );
    assert_eq!(super::gdrive_webview::SYNC_FOLDER_NAME, "Spark Backup");
    assert_eq!(
        super::gdrive_webview::build_folder_metadata_json(true)["title"],
        "Spark Backup"
    );
    assert_eq!(
        super::gdrive_webview::build_folder_metadata_json(true)["mimeType"],
        "application/vnd.google-apps.folder"
    );
    assert_eq!(
        super::gdrive_webview::build_folder_metadata_json(false)["name"],
        "Spark Backup"
    );

    let req_id = super::gdrive_webview::next_request_id();
    assert!(req_id.len() <= 24);
    let (tx, mut rx) = tokio::sync::oneshot::channel();
    gdrive_rt.pending.lock().unwrap().insert(
        req_id.clone(),
        super::gdrive_webview::PendingGDriveRequest {
            chunks: Vec::new(),
            sender: Some(tx),
        },
    );

    super::gdrive_webview::apply_bridge_navigation(
        &gdrive_rt,
        &url::Url::parse(&format!(
            "spark-internal://google-drive-sync/chunk?token={token}&request={req_id}&value=0%3A2&payload=abc"
        ))
        .unwrap(),
    );
    super::gdrive_webview::apply_bridge_navigation(
        &gdrive_rt,
        &url::Url::parse(&format!(
            "spark-internal://google-drive-sync/complete?token={token}&request={req_id}&payload=def"
        ))
        .unwrap(),
    );
    assert_eq!(rx.try_recv().unwrap(), Ok("abcdef".to_string()));
}

#[test]
fn sync_groups_filter_the_file_and_the_fingerprint() {
    use super::sync_filter::{filter, fingerprint};
    let (_db, doc) = setup();
    let all = RestoreGroup::ALL.to_vec();
    let no_notes: Vec<_> = all
        .iter()
        .copied()
        .filter(|g| *g != RestoreGroup::Notes)
        .collect();
    let mut changed_notes = doc.clone();
    changed_notes
        .data
        .notes
        .push(crate::database::notes_repo::PageNote {
            url: "https://example.test/other".into(),
            title: "Other".into(),
            text: "Only here".into(),
            updated_at: 3,
        });
    // A computer that does not sync notes sees no difference; one that does, sees it.
    assert_eq!(
        fingerprint(&doc, &no_notes).unwrap(),
        fingerprint(&changed_notes, &no_notes).unwrap()
    );
    assert_ne!(
        fingerprint(&doc, &all).unwrap(),
        fingerprint(&changed_notes, &all).unwrap()
    );
    // The device name never counts.
    let mut other_device = doc.clone();
    other_device.device = Some("NOTEBOOK".into());
    assert_eq!(
        fingerprint(&doc, &all).unwrap(),
        fingerprint(&other_device, &all).unwrap()
    );
    // Settings and shortcuts count only when synced.
    let mut changed_prefs = doc.clone();
    changed_prefs.preferences.show_bookmarks_bar = !doc.preferences.show_bookmarks_bar;
    let data_only = RestoreGroup::DATA.to_vec();
    assert_eq!(
        fingerprint(&doc, &data_only).unwrap(),
        fingerprint(&changed_prefs, &data_only).unwrap()
    );
    assert_ne!(
        fingerprint(&doc, &all).unwrap(),
        fingerprint(&changed_prefs, &all).unwrap()
    );
    // What is not synced does not leave this computer.
    let mut sent = changed_notes.clone();
    filter(&mut sent, &no_notes);
    assert!(sent.data.notes.is_empty());
    assert_eq!(
        sent.data.workspaces.len(),
        changed_notes.data.workspaces.len()
    );
    validation::document(&sent).unwrap();
}
