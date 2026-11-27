//! Commands of the profiles section in Settings. Only the trusted shell reaches them; private windows
//! refuse them (a private window is not a profile).
use crate::error_log::{self, Level};
use crate::profile::store::{self, ProfileInfo, MAX_PROFILES};
use crate::profile::{launch, launcher, status, ProfileRole};
use serde::Serialize;
use tauri::{AppHandle, Manager, State};
use zeroize::Zeroizing;

#[derive(Serialize)]
pub struct ProfileList {
    pub current: String,
    pub items: Vec<ProfileInfo>,
    pub max: usize,
    /// Ids of the profiles that ask for an app lock when they open.
    pub locked: Vec<String>,
    /// The profile Spark opens when it starts, or `None` when it asks which one to open.
    pub startup: Option<String>,
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
        let items = store::list(role.local_root());
        let locked = items
            .iter()
            .filter(|profile| {
                status::has_app_lock(&profile.id, &role.data_dir_of(&profile.id)) == Some(true)
            })
            .map(|profile| profile.id.clone())
            .collect();
        Ok(ProfileList {
            current: role.id().to_string(),
            startup: status::startup_profile(role.local_root(), &items),
            items,
            max: MAX_PROFILES,
            locked,
        })
    })
    .await
}

#[tauri::command]
pub async fn create_profile(
    app: AppHandle,
    role: State<'_, ProfileRole>,
    name: String,
    avatar: Option<String>,
    password: Option<String>,
) -> Result<ProfileInfo, String> {
    allowed(&app)?;
    let role = role.inner().clone();
    let password = password.map(Zeroizing::new);
    blocking("creating a profile", move || {
        store::create_with_options(
            role.local_root(),
            &name,
            avatar.as_deref(),
            password.as_ref().map(|value| value.as_str()),
            timestamp(),
        )
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

#[derive(Serialize)]
pub struct LaunchState {
    /// `"picker"` while this process only asks which profile to open, `"browser"` otherwise.
    pub mode: &'static str,
    /// The profile has an app lock that is still closed: the lock screen is what to show.
    pub locked: bool,
}

/// What this process is for. It answers before start-up is ready, so the first screen can choose
/// between the browser and the profile picker.
#[tauri::command]
pub fn get_launch_state(
    role: State<'_, ProfileRole>,
    lock: State<'_, crate::app_lock::AppLockService>,
) -> LaunchState {
    LaunchState {
        mode: if role.is_picker() {
            "picker"
        } else {
            "browser"
        },
        locked: lock.is_locked(),
    }
}

#[derive(Serialize)]
pub struct PickerState {
    pub profiles: Vec<ProfileInfo>,
    pub ask_every_time: bool,
    pub last_profile: Option<String>,
}

/// The list the picker shows. Only the picker process answers.
#[tauri::command]
pub async fn get_picker_state(role: State<'_, ProfileRole>) -> Result<PickerState, String> {
    require_picker(&role)?;
    let role = role.inner().clone();
    blocking("listing the profiles for the picker", move || {
        let prefs = launcher::load(role.local_root());
        Ok(PickerState {
            profiles: store::list(role.local_root()),
            ask_every_time: prefs.ask_every_time,
            last_profile: prefs.last_profile,
        })
    })
    .await
}

fn require_picker(role: &ProfileRole) -> Result<(), String> {
    if role.is_picker() {
        return Ok(());
    }
    error_log::report(
        Level::Warning,
        "profiles",
        "a picker command was refused: this process is not the profile picker",
        "",
    );
    Err("profile.not_picker".into())
}

/// Opens the chosen profile in its own process, remembers the choice, and ends the picker.
#[tauri::command]
pub async fn choose_profile(
    app: AppHandle,
    role: State<'_, ProfileRole>,
    id: String,
    ask_every_time: bool,
) -> Result<(), String> {
    require_picker(&role)?;
    let role = role.inner().clone();
    blocking("opening the chosen profile", move || {
        launcher::remember(role.local_root(), &id, ask_every_time)?;
        launch::open(&role, &id)
    })
    .await?;
    app.exit(0);
    Ok(())
}

#[derive(Serialize)]
pub struct LauncherPrefs {
    pub ask_every_time: bool,
}

#[tauri::command]
pub async fn get_launcher_prefs(
    app: AppHandle,
    role: State<'_, ProfileRole>,
) -> Result<LauncherPrefs, String> {
    allowed(&app)?;
    let role = role.inner().clone();
    blocking("reading the profile choice at start-up", move || {
        Ok(LauncherPrefs {
            ask_every_time: launcher::load(role.local_root()).ask_every_time,
        })
    })
    .await
}

/// "Ask which profile to open when Spark starts". The profile chosen last stays remembered.
#[tauri::command]
pub async fn set_ask_profile_at_startup(
    app: AppHandle,
    role: State<'_, ProfileRole>,
    enabled: bool,
) -> Result<(), String> {
    allowed(&app)?;
    let role = role.inner().clone();
    blocking("saving the profile choice at start-up", move || {
        let mut prefs = launcher::load(role.local_root());
        prefs.ask_every_time = enabled;
        launcher::save(role.local_root(), &prefs)
    })
    .await
}
