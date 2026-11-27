use super::{
    sync::{self, SyncRuntime, SyncTargetConfig},
    tests::setup,
};
use crate::database::{backup_repo, preferences_repo as prefs, session_repo};

fn webdav(remember_secret: bool) -> SyncTargetConfig {
    SyncTargetConfig::DirectWebdav {
        endpoint_url: "https://example.test/dav".into(),
        username: "fixture-user".into(),
        remember_secret,
        auto_save: false,
        apply_preferences_on_direct_pull: false,
        nextcloud_server: None,
    }
}

#[test]
fn legacy_migration_scrubs_the_deleted_cell_and_truncates_the_wal() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("fixture.sqlite");
    let conn = rusqlite::Connection::open(&path).unwrap();
    crate::database::connection::initialize_schema(&conn).unwrap();
    prefs::initialize_preferences_table(&conn).unwrap();
    let fixture = "artificial-migration-value-1794227";
    prefs::store(&conn, prefs::BACKUP_SYNC_SECRET_KEY, fixture).unwrap();
    let runtime = SyncRuntime::default();
    assert!(sync::resolve_secret(&conn, &runtime).unwrap().is_some());
    for file in [path.clone(), directory.path().join("fixture.sqlite-wal")] {
        if let Ok(bytes) = std::fs::read(file) {
            assert!(!bytes
                .windows(fixture.len())
                .any(|window| window == fixture.as_bytes()));
        }
    }
}

#[test]
fn legacy_credentials_migrate_and_secure_store_failures_never_write_plaintext() {
    let (conn, _) = setup();
    prefs::initialize_preferences_table(&conn).unwrap();
    let runtime = SyncRuntime::default();
    prefs::store(
        &conn,
        prefs::BACKUP_SYNC_SECRET_KEY,
        "fixture-legacy-password",
    )
    .unwrap();
    assert_eq!(
        sync::resolve_secret(&conn, &runtime).unwrap().as_deref(),
        Some("fixture-legacy-password")
    );
    assert!(prefs::load(&conn, prefs::BACKUP_SYNC_SECRET_KEY)
        .unwrap()
        .is_none());
    let reopened = SyncRuntime {
        credentials: runtime.credentials.clone(),
        ..Default::default()
    };
    assert_eq!(
        sync::resolve_secret(&conn, &reopened).unwrap().as_deref(),
        Some("fixture-legacy-password")
    );
    runtime
        .credentials
        .unavailable
        .store(true, std::sync::atomic::Ordering::Relaxed);
    assert!(sync::save_config(
        &conn,
        &runtime,
        webdav(true),
        Some("fixture-new-password".into())
    )
    .is_err());
    assert!(prefs::load(&conn, prefs::BACKUP_SYNC_SECRET_KEY)
        .unwrap()
        .is_none());

    let locked = SyncRuntime::default();
    locked
        .credentials
        .unavailable
        .store(true, std::sync::atomic::Ordering::Relaxed);
    prefs::store(
        &conn,
        prefs::BACKUP_SYNC_SECRET_KEY,
        "fixture-migration-password",
    )
    .unwrap();
    assert!(sync::resolve_secret(&conn, &locked).is_err());
    assert!(prefs::load(&conn, prefs::BACKUP_SYNC_SECRET_KEY)
        .unwrap()
        .is_none());
    assert_eq!(
        sync::resolve_secret(&conn, &locked).unwrap().as_deref(),
        Some("fixture-migration-password")
    );
}

#[test]
fn forgetting_credentials_removes_the_secure_entry_and_new_targets_do_not_reuse_it() {
    let (conn, _) = setup();
    prefs::initialize_preferences_table(&conn).unwrap();
    let runtime = SyncRuntime::default();
    sync::save_config(
        &conn,
        &runtime,
        webdav(true),
        Some("fixture-password".into()),
    )
    .unwrap();
    let mut other = webdav(true);
    if let SyncTargetConfig::DirectWebdav { endpoint_url, .. } = &mut other {
        *endpoint_url = "https://other.test/dav".into();
    }
    sync::save_config(&conn, &runtime, other, None).unwrap();
    assert!(sync::resolve_secret(&conn, &runtime).unwrap().is_none());
    sync::save_config(
        &conn,
        &runtime,
        webdav(true),
        Some("fixture-password".into()),
    )
    .unwrap();
    sync::save_config(&conn, &runtime, webdav(false), None).unwrap();
    let reopened = SyncRuntime {
        credentials: runtime.credentials.clone(),
        ..Default::default()
    };
    assert!(sync::resolve_secret(&conn, &reopened).unwrap().is_none());
    assert_eq!(
        sync::resolve_secret(&conn, &runtime).unwrap().as_deref(),
        Some("fixture-password")
    );
}

#[test]
fn import_transaction_preserves_local_account_identity_before_any_later_save() {
    let (conn, _) = setup();
    let tabs = session_repo::load_session_tabs(&conn).unwrap();
    let id = tabs[0].id.clone();
    conn.execute(
        "UPDATE session_tabs SET container_id='fixture-account' WHERE tab_id=?1",
        [&id],
    )
    .unwrap();
    let current = backup_repo::collect(&conn).unwrap();
    let mut merged = current.clone();
    let mut imported = merged.tabs[0].clone();
    imported.id = "tab-imported".into();
    imported.is_active = false;
    merged.tabs.push(imported);
    backup_repo::commit(&conn, &current, &merged, None).unwrap();
    let restored = session_repo::load_session_tabs(&conn).unwrap();
    assert_eq!(
        restored
            .iter()
            .find(|t| t.id == id)
            .unwrap()
            .container_id
            .as_deref(),
        Some("fixture-account")
    );
    assert!(restored
        .iter()
        .find(|t| t.id == "tab-imported")
        .unwrap()
        .container_id
        .is_none());
}
