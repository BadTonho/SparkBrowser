//! The two autofill switches of Settings → Autofill & passwords.
use crate::autofill::{self, AutofillService, Settings};
use crate::database::executor;
use tauri::{AppHandle, Manager, State};

#[tauri::command]
pub fn get_autofill_settings(service: State<'_, AutofillService>) -> Settings {
    service.settings()
}

/// Saves the switches. Turning "private windows" on makes the next private window carry a read-only
/// copy of the vault; windows already open keep what they were started with.
#[tauri::command]
pub async fn set_autofill_settings(
    app: AppHandle,
    service: State<'_, AutofillService>,
    enabled: bool,
    in_private: bool,
) -> Result<Settings, String> {
    let private = app
        .try_state::<crate::private_session::PrivateRole>()
        .is_some_and(|role| role.is_private());
    if private {
        return Err("vault.private_window".into());
    }
    let settings = Settings {
        enabled,
        in_private,
    };
    executor::run(app, move |conn| autofill::save_settings(conn, settings)).await?;
    service.apply(settings);
    Ok(settings)
}
