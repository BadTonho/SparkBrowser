//! Brings documents written by older versions up to the data shape the importer expects.
//!
//! The file `version` is the schema version. Validation runs first with each version's own
//! rules, then `upgrade` applies one step per version crossed. The document keeps its original
//! `version`, which the review panel shows. Add a step here whenever the format changes.
use super::model::*;

pub fn upgrade(document: &mut BackupDocument) {
    // 1→2 (site zoom), 2→3 (floating window shadow), 4→5 (checksum) and 5→6 (settings display) only add optional fields or
    // file-level metadata, which deserialization defaults already cover. Add a `match` here when a
    // step needs code again.
    for from in document.version..CURRENT_VERSION {
        // Split layouts gained explicit left/right tab groups; legacy ones put the rest on the left.
        if from == 3 {
            normalize_splits(&mut document.data);
        }
    }
}

fn normalize_splits(data: &mut BackupData) {
    let tabs: Vec<_> = data.tabs.iter().map(BackupTab::tab).collect();
    for layout in &mut data.split_layouts {
        layout.normalize(&tabs);
    }
}
