use super::{files, model::*, resumable::empty_hash};
use crate::database::{connection::initialize_schema, download_jobs_repo as repo};
fn record(root: &std::path::Path) -> DownloadRecord {
    DownloadRecord {
        task: DownloadTask {
            id: "download-fixture".into(),
            revision: 1,
            file_name: "fixture.bin".into(),
            url: "https://example.test/file".into(),
            source: "url".into(),
            source_page_url: None,
            status: "Paused".into(),
            received: 0,
            total: None,
            error: None,
            cancellable: true,
            can_pause: false,
            can_resume: true,
            can_restart: true,
            can_verify: false,
            save_path: Some(root.join("fixture.bin").to_string_lossy().into()),
            expected_sha256: None,
            actual_sha256: None,
            verification_received: 0,
        },
        tab_id: "tab-1".into(),
        directory: root.into(),
        confirmed: 0,
        prefix_sha256: empty_hash(),
        final_url: None,
        etag: None,
        body_completed: false,
        published: false,
    }
}
#[test]
fn journal_and_history_commit_together_without_duplicate_completion() {
    let conn = rusqlite::Connection::open_in_memory().unwrap();
    initialize_schema(&conn).unwrap();
    let root = tempfile::tempdir().unwrap();
    let mut record = record(root.path());
    repo::save(&conn, &record).unwrap();
    record.task.status = "Completed".into();
    record.published = true;
    conn.execute_batch("CREATE TRIGGER fail_job BEFORE UPDATE ON download_jobs BEGIN SELECT RAISE(ABORT,'Artificial journal failure'); END").unwrap();
    assert!(repo::complete(&conn, &record).is_err());
    assert_eq!(
        conn.query_row("SELECT COUNT(*) FROM downloads", [], |r| r.get::<_, i64>(0))
            .unwrap(),
        0
    );
    assert_eq!(repo::load(&conn).unwrap()[0].task.status, "Paused");
    conn.execute_batch("DROP TRIGGER fail_job").unwrap();
    repo::complete(&conn, &record).unwrap();
    repo::complete(&conn, &record).unwrap();
    assert_eq!(
        conn.query_row("SELECT COUNT(*) FROM downloads", [], |r| r.get::<_, i64>(0))
            .unwrap(),
        1
    );
}
#[test]
fn publishing_preserves_existing_files_and_recovers_after_rename() {
    let root = tempfile::tempdir().unwrap();
    let root = std::fs::canonicalize(root.path()).unwrap();
    let mut record = record(&root);
    let partial = files::owned_partial(&record).unwrap();
    std::fs::write(&partial, b"fixture payload").unwrap();
    let (size, hash) = files::hash(&partial, None).unwrap();
    record.confirmed = size;
    record.task.actual_sha256 = Some(hash);
    let target = std::path::Path::new(record.task.save_path.as_ref().unwrap());
    std::fs::write(target, b"existing user file").unwrap();
    assert!(files::publish(&record).is_err());
    assert_eq!(std::fs::read(target).unwrap(), b"existing user file");
    assert!(partial.exists());
    std::fs::remove_file(target).unwrap();
    files::publish(&record).unwrap();
    assert!(!partial.exists());
    files::publish(&record).unwrap();
    std::fs::write(target, b"changed").unwrap();
    assert!(files::publish(&record).is_err());
}
#[test]
fn capabilities_require_valid_identity_and_checksum_input() {
    let root = tempfile::tempdir().unwrap();
    let mut r = record(root.path());
    r.confirmed = 1;
    r.refresh_capabilities(false);
    assert!(!r.task.can_resume);
    assert!(r.task.can_restart);
    r.etag = Some("\"strong\"".into());
    r.refresh_capabilities(false);
    assert!(r.task.can_resume);
    r.task.source = "native".into();
    r.refresh_capabilities(false);
    assert!(!r.task.can_resume);
    r.refresh_capabilities(true);
    assert!(r.task.can_resume);
    assert!(checksum(Some("wrong".into())).is_err());
    assert_eq!(
        checksum(Some("A".repeat(64))).unwrap(),
        Some("a".repeat(64))
    );
    r.task.id = "../outside".into();
    assert!(files::owned_partial(&r).is_err());
}
