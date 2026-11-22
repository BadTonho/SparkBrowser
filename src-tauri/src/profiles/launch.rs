//! Opening a profile: a profile already running opens a new window, otherwise Spark starts again
//! with `--profile <id>` as a separate process (it does not wait for it).
use super::{store, ProfileRole, DEFAULT_ID, FLAG};
use crate::error_log::{self, Level};
use crate::instance_link;
use std::process::Command;

pub fn open(role: &ProfileRole, id: &str) -> Result<(), String> {
    if id != DEFAULT_ID && !store::valid_id(id) {
        return Err("profile.not_found".into());
    }
    if store::get(role.local_root(), id).is_none() {
        return Err("profile.not_found".into());
    }
    match instance_link::ask_running_instance(&role.data_dir_of(id)) {
        Ok(true) => return Ok(()),
        Ok(false) => {}
        Err(details) => error_log::report(
            Level::Warning,
            "profiles",
            "the running profile could not be asked for a window, starting it again",
            &format!("{id}: {details}"),
        ),
    }
    let exe = std::env::current_exe().map_err(|error| {
        error_log::report(
            Level::Error,
            "profiles",
            "the path of the Spark executable is unavailable",
            &error.to_string(),
        );
        "profile.open_failed".to_string()
    })?;
    Command::new(exe)
        .arg(FLAG)
        .arg(id)
        .spawn()
        .map(drop)
        .map_err(|error| {
            error_log::report(
                Level::Error,
                "profiles",
                "the profile could not be started",
                &format!("{id}: {error}"),
            );
            "profile.open_failed".to_string()
        })
}
