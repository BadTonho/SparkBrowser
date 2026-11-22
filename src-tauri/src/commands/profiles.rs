//! Commands of the profiles section in Settings. Only the trusted shell reaches them; private windows
//! refuse them (a private window is not a profile).
use crate::error_log::{self, Level};
use crate::profiles::store::{self, ProfileInfo, MAX_PROFILES};
use crate::profiles::{launch, ProfileRole};
use serde::Serialize;
use tauri::{AppHandle, Manager, State};

#[derive(Serialize)]
pub struct ProfileList {
    pub current: String,
    pub items: Vec<ProfileInfo>,
    pub max: usize,
}

fn allowed(app: &AppHandle) -> Result<(), String> {
    let private = app
        .try_state::<crate::private_session::PrivateRole>()
        .is_some_and(|role| role.is_private());
    if private {
        error_log::report(
            Level::Info,
            "profiles",
            "profiles were refused in a private window",
            "",
        );
        return Err("profile.private_window".into());
    }
    Ok(())
}

fn timestamp() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_secs() as i64)
}

/// Runs file work off the interface thread, reporting a task that stopped unexpectedly.
async fn blocking<T: Send + 'static>(
    what: &'static str,
    work: impl FnOnce() -> Result<T, String> + Send + 'static,
) -> Result<T, String> {
    tauri::async_runtime::spawn_blocking(work)
        .await
        .map_err(|error| {
            error_log::report(
                Level::Error,
                "profiles",
                &format!("{what} stopped unexpectedly"),
                &error.to_string(),
            );
            "profile.failed".to_string()
        })?
}

#[tauri::command]
pub async fn list_profiles(
    app: AppHandle,
    role: State<'_, ProfileRole>,
) -> Result<ProfileList, String> {
    allowed(&app)?;
    let role = role.inner().clone();
    blocking("listing the profiles", move || {
        Ok(ProfileList {
            current: role.id().to_string(),
            items: store::list(role.local_root()),
            max: MAX_PROFILES,
        })
    })
    .await
}

#[tauri::command]
pub async fn create_profile(
    app: AppHandle,
    role: State<'_, ProfileRole>,
    name: String,
) -> Result<ProfileInfo, String> {
    allowed(&app)?;
    let role = role.inner().clone();
    blocking("creating a profile", move || {
        store::create(role.local_root(), &name, timestamp())
    })
    .await
}

#[tauri::command]
pub async fn rename_profile(
    app: AppHandle,
    role: State<'_, ProfileRole>,
    id: String,
    name: String,
) -> Result<ProfileInfo, String> {
    allowed(&app)?;
    let role = role.inner().clone();
    let renamed = blocking("renaming a profile", move || {
        store::rename(role.local_root(), &id, &name)
    })
    .await?;
    refresh_title(&app);
    Ok(renamed)
}

#[tauri::command]
pub async fn delete_profile(
    app: AppHandle,
    role: State<'_, ProfileRole>,
    id: String,
) -> Result<(), String> {
    allowed(&app)?;
    let role = role.inner().clone();
    blocking("deleting a profile", move || {
        store::delete(role.local_root(), role.id(), &id)
    })
    .await
}

/// Opens a profile in its own window: the running one when it is open, a new process otherwise.
#[tauri::command]
pub async fn open_profile(
    app: AppHandle,
    role: State<'_, ProfileRole>,
    id: String,
) -> Result<(), String> {
    allowed(&app)?;
    let role = role.inner().clone();
    blocking("opening a profile", move || launch::open(&role, &id)).await
}

/// "Spark - Name" in the title of a profile other than the Default one, so windows of different
/// profiles can be told apart. The Default profile keeps the title it has always had.
pub fn refresh_title(app: &AppHandle) {
    let Some(role) = app.try_state::<ProfileRole>() else {
        return;
    };
    if role.is_default() {
        return;
    }
    let Some(profile) = store::get(role.local_root(), role.id()) else {
        return;
    };
    if let Some(window) = app.get_webview_window("main") {
        error_log::ignored(window.set_title(&format!("Spark - {}", profile.name)));
    }
}
