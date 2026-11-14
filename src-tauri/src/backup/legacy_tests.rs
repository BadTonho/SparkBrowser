//! Data written before the app was renamed from Veloce to Spark must keep opening.
use super::{crypto, files, local, model::*, sync, tests::setup};

#[test]
fn encrypted_backups_with_the_old_envelope_tag_still_decrypt() {
    let (_conn, doc) = setup();
    let sealed = files::encode(&doc, Some("correct horse")).unwrap();
    let mut envelope: serde_json::Value = serde_json::from_slice(&sealed).unwrap();
    assert_eq!(envelope["format"], crypto::ENCRYPTED_FORMAT);
    // Files written by the old build carried the old tag over the same ciphertext and AAD.
    envelope["format"] = crypto::LEGACY_ENCRYPTED_FORMAT.into();
    let legacy = serde_json::to_vec(&envelope).unwrap();
    assert!(crypto::is_encrypted(&legacy));
    let restored = files::decode_with(&legacy, Some("correct horse")).unwrap();
    assert_eq!(restored.data.tabs.len(), doc.data.tabs.len());
    envelope["format"] = "something-else".into();
    assert!(!crypto::is_encrypted(
        &serde_json::to_vec(&envelope).unwrap()
    ));
}

#[test]
fn documents_with_the_old_format_tag_are_read_and_new_ones_are_written_with_the_new_tag() {
    let (_conn, mut doc) = setup();
    for version in [4, CURRENT_VERSION] {
        doc.version = version;
        doc.format = LEGACY_FORMAT.into();
        let bytes = files::encode(&doc, None).unwrap();
        assert_eq!(
            files::decode(&bytes).unwrap().format,
            LEGACY_FORMAT,
            "v{version}"
        );
    }
    doc.format = FORMAT.into();
    let written: serde_json::Value =
        serde_json::from_slice(&files::encode(&doc, None).unwrap()).unwrap();
    assert_eq!(written["format"], "spark-backup");
    doc.format = "other-app-backup".into();
    assert!(files::encode(&doc, None).is_err());
}

#[test]
fn history_files_keep_both_suffixes_and_new_snapshots_use_the_new_one() {
    assert!(local::file_name_for_test(1_700_000_000, "manual").ends_with(".spark-backup.json"));
    for name in [
        "backup-1700000000-manual.spark-backup.json",
        "backup-1700000000-scheduled.veloce-backup.json",
    ] {
        assert!(local::parses_for_test(name), "{name}");
    }
    assert!(!local::parses_for_test(
        "backup-1700000000-manual.other-backup.json"
    ));
    assert!(!local::parses_for_test(
        "../backup-1-manual.veloce-backup.json"
    ));
}

#[test]
fn folder_pulls_fall_back_to_the_old_file_name_only_when_the_new_one_is_missing() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().to_str().unwrap();
    let new = dir.path().join(sync::SYNC_FILE_NAME);
    let old = dir.path().join(sync::LEGACY_SYNC_FILE_NAME);
    assert_eq!(
        sync::resolve_folder_pull_path(path).unwrap(),
        new,
        "nothing yet: the new name"
    );
    std::fs::write(&old, "{}").unwrap();
    assert_eq!(sync::resolve_folder_pull_path(path).unwrap(), old);
    std::fs::write(&new, "{}").unwrap();
    assert_eq!(
        sync::resolve_folder_pull_path(path).unwrap(),
        new,
        "the new file wins"
    );
    // Pushes always target the new name.
    assert_eq!(sync::resolve_folder_file_path(path).unwrap(), new);
}

#[test]
fn webdav_derives_the_old_file_url_only_from_a_directory_endpoint() {
    let derived = sync::resolve_webdav_file_url("https://cloud.example.test/dav/backups/").unwrap();
    assert!(derived.path().ends_with("/browser.spark-backup.json"));
    let legacy = sync::legacy_webdav_file_url_for_test(&derived).unwrap();
    assert!(legacy.path().ends_with("/browser.veloce-backup.json"));
    assert_eq!(legacy.host_str(), derived.host_str());
    // An endpoint that names its own file is used as given, in either spelling.
    for explicit in [
        "https://cloud.example.test/dav/mine.spark-backup.json",
        "https://cloud.example.test/dav/mine.veloce-backup.json",
    ] {
        let url = sync::resolve_webdav_file_url(explicit).unwrap();
        assert_eq!(url.as_str(), explicit);
        assert!(sync::legacy_webdav_file_url_for_test(&url).is_none());
    }
}

#[test]
fn the_old_new_tab_address_is_recognised_and_rewritten_on_import() {
    use crate::window::new_tab::{is_new_tab, LEGACY_URL, URL};
    assert!(is_new_tab(URL) && is_new_tab(LEGACY_URL));
    assert!(!is_new_tab("veloce://newtab/") && !is_new_tab("spark://newtab?x=1"));
    assert_eq!(super::validation::tab_url(LEGACY_URL).unwrap(), URL);
    assert_eq!(super::validation::tab_url(URL).unwrap(), URL);
}
