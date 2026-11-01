#[cfg(windows)]
mod frames;
use serde::{Deserialize, Serialize};
use std::sync::{atomic::AtomicU64, Mutex};
#[cfg(windows)]
pub mod profile;
#[cfg(windows)]
pub mod requests;
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PermissionKind {
    Camera,
    Microphone,
    Location,
    Notifications,
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PermissionState {
    Ask,
    Allow,
    Block,
}
#[derive(Clone, Serialize)]
pub struct SitePermission {
    pub origin: String,
    pub kind: PermissionKind,
    pub state: PermissionState,
}
#[derive(Clone, Serialize)]
pub struct PermissionRequest {
    pub id: String,
    pub tab_id: String,
    pub origin: String,
    pub top_origin: String,
    pub frame_origin: Option<String>,
    pub kind: PermissionKind,
}
#[derive(Default)]
pub struct PermissionRequests {
    pub next: AtomicU64,
    pub pending: Mutex<Vec<PermissionRequest>>,
}
impl PermissionRequests {
    pub fn admit(&self, request: PermissionRequest) -> bool {
        let Ok(mut pending) = self.pending.lock() else {
            return false;
        };
        if pending.len() >= 16 {
            return false;
        }
        pending.push(request);
        true
    }
}
#[derive(Serialize)]
pub struct PermissionSnapshot {
    pub supported: bool,
    pub message: String,
    pub settings: Vec<SitePermission>,
    pub requests: Vec<PermissionRequest>,
}
pub fn origin(raw: &str) -> Result<String, String> {
    let url = url::Url::parse(raw).map_err(|_| "Invalid site origin")?;
    if !matches!(url.scheme(), "http" | "https")
        || url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
    {
        return Err("Site permissions require an HTTP/HTTPS origin".into());
    }
    Ok(url.origin().ascii_serialization())
}
pub fn cancel_view(app: &tauri::AppHandle, label: &str) {
    #[cfg(windows)]
    {
        let (app, label) = (app.clone(), label.to_owned());
        let dispatch = app.clone();
        let _ = dispatch.run_on_main_thread(move || requests::cancel(&app, &label));
    }
    #[cfg(not(windows))]
    let _ = (app, label);
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pending_permission_metadata_is_bounded() {
        let registry = PermissionRequests::default();
        for index in 0..17 {
            let admitted = registry.admit(PermissionRequest {
                id: index.to_string(),
                tab_id: "fixture".into(),
                origin: "https://frame.test".into(),
                top_origin: "https://page.test".into(),
                frame_origin: None,
                kind: PermissionKind::Microphone,
            });
            assert_eq!(admitted, index < 16);
        }
        assert_eq!(registry.pending.lock().unwrap().len(), 16);
    }
    #[test]
    fn origins_preserve_scheme_and_port_without_path_or_fragment() {
        assert_eq!(
            origin("https://EXAMPLE.test:8443/a?q=1#x").unwrap(),
            "https://example.test:8443"
        );
        assert!(origin("file:///private").is_err());
        assert!(origin("https://user:password@example.test/").is_err());
    }
}
