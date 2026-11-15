//! Checksum verification, newer-version rejection and the version upgrade chain.
use super::{files, migrate, model::*, tests::setup, validation};
use crate::state::split_layout::SplitLayout;

fn encoded(version: u32) -> (Vec<u8>, BackupDocument) {
    let (_conn, mut doc) = setup();
    doc.version = version;
    (files::encode(&doc, None).unwrap(), doc)
}
fn as_value(bytes: &[u8]) -> serde_json::Value {
    serde_json::from_slice(bytes).unwrap()
}

#[test]
fn current_files_carry_a_checksum_and_roundtrip() {
    let (bytes, doc) = encoded(CURRENT_VERSION);
    let value = as_value(&bytes);
    let checksum = value["checksum"].as_str().unwrap();
    assert_eq!(checksum.len(), 64);
    assert!(checksum.bytes().all(|b| b.is_ascii_hexdigit()));
    let restored = files::decode(&bytes).unwrap();
    assert_eq!(restored.version, CURRENT_VERSION);
    assert_eq!(restored.data.tabs.len(), doc.data.tabs.len());
}

#[test]
fn edited_content_is_reported_as_corrupted() {
    let (bytes, _) = encoded(CURRENT_VERSION);
    let mut value = as_value(&bytes);
    value["data"]["tabs"][0]["title"] = "Edited by hand".into();
    let tampered = serde_json::to_vec(&value).unwrap();
    assert_eq!(
        files::decode(&tampered).err().unwrap(),
        "backup.checksum_mismatch"
    );
    let mut value = as_value(&bytes);
    value["preferences"]["resident_limit"] = 50.into();
    assert_eq!(
        files::decode(&serde_json::to_vec(&value).unwrap())
            .err()
            .unwrap(),
        "backup.checksum_mismatch"
    );
}

#[test]
fn reformatting_does_not_break_the_checksum() {
    let (bytes, _) = encoded(CURRENT_VERSION);
    let compact = serde_json::to_vec(&as_value(&bytes)).unwrap();
    assert!(files::decode(&compact).is_ok());
    let mut spaced = b"\n\n  ".to_vec();
    spaced.extend_from_slice(&bytes);
    assert!(files::decode(&spaced).is_ok());
}

#[test]
fn a_current_file_without_checksum_or_with_a_bad_one_is_rejected() {
    let (bytes, _) = encoded(CURRENT_VERSION);
    let mut value = as_value(&bytes);
    value.as_object_mut().unwrap().remove("checksum");
    assert_eq!(
        files::decode(&serde_json::to_vec(&value).unwrap())
            .err()
            .unwrap(),
        "backup.checksum_missing"
    );
    let mut value = as_value(&bytes);
    value["checksum"] = 12.into();
    assert_eq!(
        files::decode(&serde_json::to_vec(&value).unwrap())
            .err()
            .unwrap(),
        "backup.checksum_mismatch"
    );
}

#[test]
fn older_versions_stay_readable_with_their_original_checksum_rules() {
    for version in 1..CURRENT_VERSION {
        let (bytes, _) = encoded(version);
        assert_eq!(as_value(&bytes).get("checksum").is_some(), version >= 5, "v{version}");
        assert_eq!(files::decode(&bytes).unwrap().version, version);
    }
}

#[test]
fn legacy_v5_without_settings_display_keeps_its_checksum_and_defaults_to_popup() {
    use sha2::{Digest, Sha256};
    let (_, mut doc) = setup();
    doc.version = 5;
    let mut value = serde_json::to_value(doc).unwrap();
    value["preferences"].as_object_mut().unwrap().remove("settings_display");
    let hash = Sha256::digest(serde_json::to_vec(&value).unwrap());
    let checksum: String = hash.iter().map(|byte| format!("{byte:02x}")).collect();
    value["checksum"] = checksum.into();
    let restored = files::decode(&serde_json::to_vec(&value).unwrap()).unwrap();
    assert_eq!(restored.preferences.settings_display, "popup");
    value["preferences"]["settings_display"] = "tab".into();
    assert_eq!(files::decode(&serde_json::to_vec(&value).unwrap()).err().unwrap(), "backup.checksum_mismatch");
}

#[test]
fn files_from_a_newer_app_get_a_specific_error_before_strict_parsing() {
    let (bytes, _) = encoded(CURRENT_VERSION);
    let mut value = as_value(&bytes);
    value["version"] = (CURRENT_VERSION + 1).into();
    value["field_from_the_future"] = true.into();
    assert_eq!(
        files::decode(&serde_json::to_vec(&value).unwrap())
            .err()
            .unwrap(),
        "backup.newer_version"
    );
    // A foreign file is not blamed on the version.
    let mut foreign = as_value(&bytes);
    foreign["format"] = "something-else".into();
    foreign["version"] = 99.into();
    assert_ne!(
        files::decode(&serde_json::to_vec(&foreign).unwrap())
            .err()
            .unwrap(),
        "backup.newer_version"
    );
}

#[test]
fn encrypted_files_verify_the_checksum_after_decryption() {
    let (_conn, doc) = setup();
    let mut current = doc.clone();
    current.version = CURRENT_VERSION;
    let sealed = files::encode(&current, Some("correct horse")).unwrap();
    assert!(files::decode_with(&sealed, Some("correct horse")).is_ok());
}

#[test]
fn upgrade_fills_split_groups_of_pre_v4_documents_and_keeps_their_version() {
    let (_conn, mut doc) = setup();
    doc.version = 3;
    doc.data.split_layouts.push(SplitLayout {
        left_tab_ids: Vec::new(),
        right_tab_ids: Vec::new(),
        workspace_id: "ws-personal".into(),
        left_tab_id: "tab-1".into(),
        right_tab_id: "tab-2".into(),
        focused_tab_id: "tab-2".into(),
        ratio: 0.4,
    });
    validation::document(&doc).unwrap();
    migrate::upgrade(&mut doc);
    assert_eq!(doc.data.split_layouts[0].left_tab_ids, ["tab-1"]);
    assert_eq!(doc.data.split_layouts[0].right_tab_ids, ["tab-2"]);
    assert_eq!(doc.version, 3);
    // Upgrading again changes nothing, and the result satisfies the strictest rules.
    let once = serde_json::to_value(&doc).unwrap();
    migrate::upgrade(&mut doc);
    assert_eq!(serde_json::to_value(&doc).unwrap(), once);
    let tabs: Vec<_> = doc.data.tabs.iter().map(BackupTab::tab).collect();
    assert!(doc.data.split_layouts[0].valid_groups(&tabs));
}

#[test]
fn upgrade_leaves_current_documents_untouched() {
    let (_conn, mut doc) = setup();
    doc.version = CURRENT_VERSION;
    let before = serde_json::to_value(&doc).unwrap();
    migrate::upgrade(&mut doc);
    assert_eq!(serde_json::to_value(&doc).unwrap(), before);
}
