//! "What syncs": the groups this computer sends and accepts. A file sent from here only carries the
//! data of its groups, and fingerprints only cover them, so two computers that sync different groups
//! still agree when nothing they share changed.
use super::model::{BackupDocument, RestoreGroup};
use sha2::{Digest, Sha256};

/// Empties the data of the groups that are not synced. Preferences stay in the file (the format needs
/// them); whether they are applied is decided by the receiving computer's own groups.
pub fn filter(doc: &mut BackupDocument, groups: &[RestoreGroup]) {
    let data = &mut doc.data;
    if !groups.contains(&RestoreGroup::Workspaces) {
        data.workspaces.clear();
        data.tabs.clear();
        data.split_layouts.clear();
    }
    if !groups.contains(&RestoreGroup::Bookmarks) {
        data.bookmarks.bookmarks.clear();
        data.bookmarks.folders.clear();
    }
    if !groups.contains(&RestoreGroup::Sessions) {
        data.sessions.clear();
    }
    if !groups.contains(&RestoreGroup::Notes) {
        data.notes.clear();
    }
    if !groups.contains(&RestoreGroup::SiteSearch) {
        data.site_search.clear();
    }
}

/// The fingerprint of what this computer syncs in `doc`: its groups' data, its settings and its
/// shortcuts only when those groups are synced. The device name never counts.
pub fn fingerprint(doc: &BackupDocument, groups: &[RestoreGroup]) -> Result<String, String> {
    let mut copy = doc.clone();
    filter(&mut copy, groups);
    let mut preferences = serde_json::to_value(&copy.preferences).map_err(|e| e.to_string())?;
    let shortcuts = preferences
        .as_object_mut()
        .and_then(|object| object.remove("keyboard_shortcuts"))
        .unwrap_or(serde_json::Value::Null);
    let settings = if groups.contains(&RestoreGroup::Settings) {
        preferences
    } else {
        serde_json::Value::Null
    };
    let shortcuts = if groups.contains(&RestoreGroup::Shortcuts) {
        shortcuts
    } else {
        serde_json::Value::Null
    };
    let bytes =
        serde_json::to_vec(&(&copy.data, settings, shortcuts)).map_err(|e| e.to_string())?;
    Ok(Sha256::digest(&bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect())
}
