use super::{merge, model::*, validation};
use crate::{
    database::{backup_repo, navigation_writer::NavigationWriter},
    state::browser_state::BrowserState,
};
use sha2::{Digest, Sha256};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Manager};

pub struct PendingPreview {
    fingerprint: [u8; 32],
    preview: BackupPreview,
    plan: merge::MergePlan,
}
#[derive(Default)]
pub struct BackupService {
    pub operation: tauri::async_runtime::Mutex<()>,
    pub sync: super::sync::SyncRuntime,
    preview: Mutex<Option<PendingPreview>>,
    source: Mutex<Option<(String, Arc<BackupDocument>)>>,
    /// Encrypted file waiting for a password. Replaced by any new selection.
    locked: Mutex<Option<Vec<u8>>>,
}
pub async fn flush(app: &AppHandle) -> Result<(), String> {
    let app = app.clone();
    tauri::async_runtime::spawn_blocking(move || app.state::<NavigationWriter>().flush())
        .await
        .map_err(|e| e.to_string())?
}
pub fn current_preferences(
    conn: &rusqlite::Connection,
    mut p: BackupPreferences,
) -> Result<BackupPreferences, String> {
    validation::preferences(&p)?;
    p.site_zoom = crate::database::site_zoom_repo::list(conn)?;
    p.update_channel = crate::database::updates_repo::load(conn, p.update_channel)?.0;
    Ok(p)
}
/// Stream canonical serialization into a digest without retaining a second document.
pub(crate) fn fingerprint(
    data: &BackupData,
    prefs: &BackupPreferences,
) -> Result<[u8; 32], String> {
    struct HashWriter(Sha256);
    impl std::io::Write for HashWriter {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            self.0.update(bytes);
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let mut writer = HashWriter(Sha256::new());
    serde_json::to_writer(&mut writer, &(data, prefs)).map_err(|e| e.to_string())?;
    Ok(writer.0.finalize().into())
}
impl BackupService {
    pub fn select(&self, document: BackupDocument) -> Result<String, String> {
        let token = crate::window::webview_manager::create_internal_bridge_token();
        *self.source.lock().map_err(|e| e.to_string())? = Some((token.clone(), Arc::new(document)));
        *self.preview.lock().map_err(|e| e.to_string())? = None;
        *self.locked.lock().map_err(|e| e.to_string())? = None;
        Ok(token)
    }
    /// Keeps an encrypted file until `unlock` succeeds or the prompt is cancelled.
    pub fn hold_locked(&self, bytes: Vec<u8>) -> Result<(), String> {
        *self.source.lock().map_err(|e| e.to_string())? = None;
        *self.preview.lock().map_err(|e| e.to_string())? = None;
        *self.locked.lock().map_err(|e| e.to_string())? = Some(bytes);
        Ok(())
    }
    /// A wrong password keeps the file locked so the user can retry.
    pub fn unlock(&self, password: &str) -> Result<String, String> {
        let bytes = self
            .locked
            .lock()
            .map_err(|e| e.to_string())?
            .clone()
            .ok_or("backup.preview_missing")?;
        let document = super::files::decode_with(&bytes, Some(password))?;
        self.select(document)
    }
    pub fn cancel_unlock(&self) -> Result<(), String> {
        *self.locked.lock().map_err(|e| e.to_string())? = None;
        Ok(())
    }
    pub fn discard(&self, token: &str) -> Result<(), String> {
        let mut source = self.source.lock().map_err(|e| e.to_string())?;
        if source.as_ref().is_some_and(|(id, _)| id == token) {
            *source = None;
            *self.preview.lock().map_err(|e| e.to_string())? = None;
        }
        Ok(())
    }
}
pub async fn capture(app: &AppHandle, prefs: BackupPreferences) -> Result<BackupDocument, String> {
    flush(app).await?;
    let state = app.state::<BrowserState>();
    let _operation = state.session().operation().await;
    state.persist_session()?;
    let db = state
        .infrastructure()
        .db()
        .lock()
        .map_err(|e| e.to_string())?;
    let conn = db.as_ref().ok_or("backup.database_unavailable")?;
    Ok(BackupDocument {
        format: FORMAT.into(),
        version: 4,
        created_at: crate::state::tab_model::TabInfo::current_epoch_secs() as i64,
        data: backup_repo::collect(conn)?,
        preferences: current_preferences(conn, prefs)?,
    })
}
pub async fn preview(
    app: &AppHandle,
    document: Option<BackupDocument>,
    source_token: Option<String>,
    prefs: BackupPreferences,
) -> Result<BackupPreview, String> {
    let service = app.state::<BackupService>();
    let _serial = service.operation.lock().await;
    // Structured imports remain supported; file selections never round-trip through JS.
    let doc = match (document, source_token) {
        (Some(doc), None) => {
            validation::document(&doc)?;
            if serde_json::to_vec(&doc).map_err(|e| e.to_string())?.len() as u64 > MAX_BYTES {
                return Err("backup.file_too_large".into());
            }
            Arc::new(doc)
        }
        (None, Some(token)) => service
            .source
            .lock()
            .map_err(|e| e.to_string())?
            .as_ref()
            .filter(|(id, _)| id == &token)
            .map(|(_, doc)| doc.clone())
            .ok_or("backup.preview_missing")?,
        _ => return Err("backup.preview_missing".into()),
    };
    let current = capture(app, prefs).await?;
    let token = crate::window::webview_manager::create_internal_bridge_token();
    let counter = *app
        .state::<BrowserState>()
        .session()
        .next_tab_counter()
        .lock()
        .map_err(|e| e.to_string())?;
    let plan = merge::plan(&current.data, &doc.data, &token, counter)?;
    let preview = BackupPreview {
        token,
        counts: merge::counts(&doc.data),
        added: plan.added.clone(),
        skipped: plan.skipped.clone(),
        preferences: doc.preferences.clone(),
    };
    *service.preview.lock().map_err(|e| e.to_string())? = Some(PendingPreview {
        fingerprint: fingerprint(&current.data, &current.preferences)?,
        preview: preview.clone(),
        plan,
    });
    Ok(preview)
}
pub async fn import(
    app: &AppHandle,
    token: String,
    apply_preferences: bool,
    prefs: BackupPreferences,
) -> Result<BackupImportResult, String> {
    let service = app.state::<BackupService>();
    let _serial = service.operation.lock().await;
    flush(app).await?;
    let state = app.state::<BrowserState>();
    let _operation = state.session().operation().await;
    state.persist_session()?;
    let mut preview_guard = service.preview.lock().map_err(|e| e.to_string())?;
    let pending = preview_guard
        .as_ref()
        .filter(|p| p.preview.token == token)
        .ok_or("backup.preview_missing")?;
    let original_tabs = state.snapshot_tabs();
    let mut db = state
        .infrastructure()
        .db()
        .lock()
        .map_err(|e| e.to_string())?;
    let conn = db.as_mut().ok_or("backup.database_unavailable")?;
    let current = backup_repo::collect(conn)?;
    let prefs = current_preferences(conn, prefs)?;
    if pending.fingerprint != fingerprint(&current, &prefs)? {
        return Err("backup.preview_stale".into());
    }
    let task = if apply_preferences {
        if backup_repo::pending(conn)?.is_some() {
            return Err("backup.preferences_pending".into());
        }
        Some(PreferenceTask {
            id: token.clone(),
            preferences: pending.preview.preferences.clone(),
            error: None,
        })
    } else {
        None
    };
    let tabs = merge_live_tabs(original_tabs, &pending.plan.data.tabs)?;
    backup_repo::commit(conn, &current, &pending.plan.data, task.as_ref())?;
    state.install_imported_session(
        tabs,
        pending.plan.data.workspaces.clone(),
        &pending.plan.data.split_layouts,
        pending.plan.next_tab_counter,
    )?;
    state.refresh_visible_tabs()?;
    let snapshot = state.snapshot_packet();
    let result = BackupImportResult {
        added: pending.preview.added.clone(),
        skipped: pending.preview.skipped.clone(),
        pending_preferences: task,
        snapshot,
        workspaces: pending.plan.data.workspaces.clone(),
    };
    *preview_guard = None;
    drop(preview_guard);
    *service.source.lock().map_err(|e| e.to_string())? = None;
    let _ = crate::security::emit_shell(app, "tabs-updated", &result.snapshot);
    Ok(result)
}

/// Import adds persistent tabs without replacing live disposable documents or
/// their position, activity, identity, selection and split membership.
fn merge_live_tabs(
    mut live: Vec<crate::state::tab_model::TabInfo>,
    merged: &[BackupTab],
) -> Result<Vec<crate::state::tab_model::TabInfo>, String> {
    let existing: std::collections::HashMap<_, _> = live
        .iter()
        .map(|tab| (tab.id.clone(), tab.is_ephemeral))
        .collect();
    for row in merged {
        match existing.get(&row.id) {
            Some(true) => return Err("backup.preview_stale".into()),
            Some(false) => {}
            None => live.push(row.tab()),
        }
    }
    Ok(live)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn live_import_keeps_disposable_tabs_order_activity_and_identity() {
        use crate::state::tab_model::TabInfo;
        let mut regular = TabInfo::new(
            "tab-1".into(),
            "https://example.test/".into(),
            "ws-personal".into(),
            false,
        );
        regular.container_id = Some("account-fixture".into());
        let mut disposable = TabInfo::new_ephemeral(
            "tab-2".into(),
            "https://example.test/private".into(),
            "ws-personal".into(),
            true,
        );
        disposable.activity.dirty_form = true;
        let mut imported = BackupTab::from(&regular);
        imported.id = "tab-3".into();
        let merged = vec![BackupTab::from(&regular), imported];
        let tabs = merge_live_tabs(vec![regular, disposable], &merged).unwrap();
        assert_eq!(
            tabs.iter().map(|t| t.id.as_str()).collect::<Vec<_>>(),
            ["tab-1", "tab-2", "tab-3"]
        );
        assert_eq!(tabs[0].container_id.as_deref(), Some("account-fixture"));
        assert!(tabs[1].is_ephemeral && tabs[1].is_active && tabs[1].activity.dirty_form);
        assert!(tabs[2].is_hibernated && !tabs[2].is_active);
        let mut collision = merged[1].clone();
        collision.id = "tab-2".into();
        assert!(merge_live_tabs(tabs, &[collision]).is_err());
    }

    #[test]
    fn streamed_fingerprint_matches_serialization_and_detects_changes() {
        let (_, mut doc) = crate::backup::tests::setup();
        let expected: [u8; 32] =
            Sha256::digest(serde_json::to_vec(&(&doc.data, &doc.preferences)).unwrap()).into();
        assert_eq!(fingerprint(&doc.data, &doc.preferences).unwrap(), expected);
        doc.data.tabs[0].url.push_str("?changed");
        assert_ne!(fingerprint(&doc.data, &doc.preferences).unwrap(), expected);
        let data_changed = fingerprint(&doc.data, &doc.preferences).unwrap();
        doc.preferences.resident_limit += 1;
        assert_ne!(
            fingerprint(&doc.data, &doc.preferences).unwrap(),
            data_changed
        );
    }

    #[test]
    fn replacing_or_discarding_a_selection_releases_its_document() {
        let (_, doc) = crate::backup::tests::setup();
        let service = BackupService::default();
        let old = service.select(doc.clone()).unwrap();
        let weak = Arc::downgrade(&service.source.lock().unwrap().as_ref().unwrap().1);
        let next = service.select(doc).unwrap();
        assert!(weak.upgrade().is_none());
        assert_ne!(old, next);
        service.discard(&old).unwrap();
        assert!(service.source.lock().unwrap().is_some());
        let weak = Arc::downgrade(&service.source.lock().unwrap().as_ref().unwrap().1);
        service.discard(&next).unwrap();
        assert!(weak.upgrade().is_none());
        assert!(service.source.lock().unwrap().is_none());
    }
}
