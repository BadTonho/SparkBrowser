#[cfg(target_os = "linux")]
pub mod linux;
#[cfg(windows)]
pub mod windows;
use crate::{database::site_zoom_repo as repo, state::browser_state::BrowserState};
pub use repo::SiteZoom;
use serde::Serialize;
use std::sync::Mutex;
use tauri::{AppHandle, Manager};
#[derive(Clone, Default, Serialize)]
pub struct ZoomSnapshot {
    pub revision: u64,
    pub sites: Vec<SiteZoom>,
}
#[derive(Default)]
pub struct ZoomService {
    inner: Mutex<ZoomSnapshot>,
}
impl ZoomService {
    pub fn initialize(&self, app: &AppHandle) -> Result<(), String> {
        let browser = app.state::<BrowserState>();
        let db = browser
            .infrastructure()
            .db()
            .lock()
            .map_err(|e| e.to_string())?;
        let sites = repo::list(db.as_ref().ok_or("zoom.database_unavailable")?)?;
        let mut inner = self.inner.lock().map_err(|e| e.to_string())?;
        inner.sites = sites;
        inner.revision += 1;
        Ok(())
    }
    pub fn snapshot(&self) -> Result<ZoomSnapshot, String> {
        Ok(self.inner.lock().map_err(|e| e.to_string())?.clone())
    }
    pub fn factor(&self, raw: &str) -> f64 {
        repo::origin(raw)
            .ok()
            .and_then(|origin| {
                self.inner
                    .lock()
                    .ok()?
                    .sites
                    .iter()
                    .find(|s| s.origin == origin)
                    .map(|s| s.factor)
            })
            .unwrap_or(1.0)
    }
    /// Caller serializes tab operations; never waits for the UI while holding database locks.
    pub fn set(&self, app: &AppHandle, row: SiteZoom) -> Result<ZoomSnapshot, String> {
        repo::validate(&row)?;
        {
            let state = app.state::<BrowserState>();
            let db = state
                .infrastructure()
                .db()
                .lock()
                .map_err(|e| e.to_string())?;
            repo::set(db.as_ref().ok_or("zoom.database_unavailable")?, &row)?;
        }
        self.initialize(app)?;
        self.apply_all(app)?;
        self.publish(app)
    }
    pub fn publish(&self, app: &AppHandle) -> Result<ZoomSnapshot, String> {
        let s = self.snapshot()?;
        crate::error_log::ignored(crate::security::emit_shell(app, "site-zoom-state", &s));
        Ok(s)
    }
    pub fn apply_all(&self, app: &AppHandle) -> Result<(), String> {
        for tab in app.state::<BrowserState>().snapshot_tabs() {
            apply(
                app,
                &crate::state::tab_model::webview_label_for_tab(&tab.id),
                &tab.url,
            )?;
        }
        Ok(())
    }
    pub fn import_pending(&self, app: &AppHandle, id: &str) -> Result<ZoomSnapshot, String> {
        {
            let state = app.state::<BrowserState>();
            let db = state
                .infrastructure()
                .db()
                .lock()
                .map_err(|e| e.to_string())?;
            let conn = db.as_ref().ok_or("zoom.database_unavailable")?;
            let pending = crate::database::backup_repo::pending(conn)?
                .filter(|p| p.id == id)
                .ok_or("backup.preference_task_missing")?;
            let tx = conn.unchecked_transaction().map_err(|e| e.to_string())?;
            repo::merge(&tx, &pending.preferences.site_zoom)?;
            tx.commit().map_err(|e| e.to_string())?;
        }
        self.initialize(app)?;
        self.apply_all(app)?;
        self.publish(app)
    }
}
pub fn apply(app: &AppHandle, label: &str, url: &str) -> Result<(), String> {
    if let Some(view) = app.get_webview(label) {
        let factor = app.state::<ZoomService>().factor(url);
        #[cfg(windows)]
        windows::apply(&view, url, factor)?;
        #[cfg(not(windows))]
        view.set_zoom(factor).map_err(|e| e.to_string())?;
    }
    Ok(())
}
