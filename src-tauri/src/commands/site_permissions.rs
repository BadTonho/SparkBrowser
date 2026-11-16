use crate::site_permissions::{PermissionKind, PermissionSnapshot, PermissionState};
use tauri::AppHandle;
#[tauri::command]
pub async fn get_site_permissions(app: AppHandle) -> PermissionSnapshot {
    #[cfg(windows)]
    {
        crate::site_permissions::profile::snapshot(&app).await
    }
    #[cfg(target_os = "linux")]
    {
        crate::site_permissions::linux::snapshot(&app).await
    }
    #[cfg(not(any(windows, target_os = "linux")))]
    {
        let _ = app;
        PermissionSnapshot {
            supported: false,
            message: "Site permission management is available on Windows WebView2 only.".into(),
            settings: vec![],
            requests: vec![],
        }
    }
}
#[tauri::command]
pub async fn set_site_permission(
    app: AppHandle,
    origin: String,
    kind: PermissionKind,
    state: PermissionState,
) -> Result<(), String> {
    #[cfg(windows)]
    {
        crate::site_permissions::profile::set(&app, origin, kind, state).await
    }
    #[cfg(target_os = "linux")]
    {
        crate::site_permissions::linux::set(&app, origin, kind, state).await
    }
    #[cfg(not(any(windows, target_os = "linux")))]
    {
        let _ = (app, origin, kind, state);
        Err("Site permission management is unavailable on this platform".into())
    }
}
#[tauri::command]
pub async fn respond_site_permission(
    app: AppHandle,
    id: String,
    decision: String,
) -> Result<(), String> {
    #[cfg(windows)]
    {
        crate::site_permissions::requests::respond(&app, id, decision).await
    }
    #[cfg(target_os = "linux")]
    {
        crate::site_permissions::linux::respond(&app, id, decision).await
    }
    #[cfg(not(any(windows, target_os = "linux")))]
    {
        let _ = (app, id, decision);
        Err("Site permission management is unavailable on this platform".into())
    }
}
