//! An explicitly selected, persistent cookie profile and Google account index.
use super::*;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

const KEY: &str = "backup_sync_google_session_v2";
const LOGIN: &str = "backup.sync_gdrive_login_required";

#[derive(Clone, Serialize, Deserialize)]
struct Binding {
    directory: Option<PathBuf>,
    authuser: String,
    email: Option<String>,
}

impl Binding {
    fn accept_email(&mut self, email: Option<&str>) -> Result<(), String> {
        let email = email.filter(|value| !value.is_empty()).ok_or(LOGIN)?;
        if self
            .email
            .as_deref()
            .is_some_and(|expected| !expected.eq_ignore_ascii_case(email))
        {
            return Err(LOGIN.into());
        }
        self.email = Some(email.to_owned());
        Ok(())
    }
}

fn select_profile_view(
    tabs: Vec<crate::state::tab_model::TabInfo>,
    directory: &Option<PathBuf>,
    mut profile: impl FnMut(&str) -> Result<Option<PathBuf>, String>,
    mut resident: impl FnMut(&str) -> bool,
) -> Option<String> {
    tabs.into_iter()
        .find(|tab| {
            !tab.is_ephemeral
                && resident(&tab.id)
                && profile(&tab.id).is_ok_and(|actual| &actual == directory)
        })
        .map(|tab| tab.id)
}

/// Called only when the shell supplies the tab opened by account selection.
/// The caller serializes tab operations so the profile cannot change here.
pub(crate) fn bind(app: &AppHandle, id: &str) -> Result<(), String> {
    let state = app.state::<BrowserState>();
    let tab = state
        .snapshot_tabs()
        .into_iter()
        .find(|t| t.id == id)
        .ok_or(LOGIN)?;
    let authuser = selected_account(&tab)?;
    if app
        .get_webview(&crate::state::tab_model::webview_label_for_tab(id))
        .is_none()
    {
        return Err(LOGIN.into());
    }
    let binding = Binding {
        directory: crate::window::workspace_data::content_directory(app, id)?,
        authuser,
        email: None,
    };
    let db = state
        .infrastructure()
        .db()
        .lock()
        .map_err(|e| e.to_string())?;
    let conn = db.as_ref().ok_or(LOGIN)?;
    preferences_repo::store(
        conn,
        KEY,
        &serde_json::to_string(&binding).map_err(|_| LOGIN)?,
    )?;
    preferences_repo::remove(conn, preferences_repo::BACKUP_SYNC_GDRIVE_EMAIL_KEY)?;
    preferences_repo::remove(conn, preferences_repo::BACKUP_SYNC_FINGERPRINT_KEY)?;
    let service = app.state::<BackupService>();
    *service
        .sync
        .gdrive
        .connected_email
        .lock()
        .map_err(|_| LOGIN)? = None;
    Ok(())
}

fn selected_account(tab: &crate::state::tab_model::TabInfo) -> Result<String, String> {
    let url = Url::parse(&tab.url).map_err(|_| LOGIN)?;
    if tab.is_ephemeral
        || url.scheme() != "https"
        || url.port_or_known_default() != Some(443)
        || !url.username().is_empty()
        || url.password().is_some()
    {
        return Err(LOGIN.into());
    }
    extract_authuser_from_url(url.as_str()).ok_or(LOGIN.into())
}

fn load(app: &AppHandle) -> Result<Binding, String> {
    let state = app.state::<BrowserState>();
    let db = state
        .infrastructure()
        .db()
        .lock()
        .map_err(|e| e.to_string())?;
    let raw = preferences_repo::load(db.as_ref().ok_or(LOGIN)?, KEY)?.ok_or(LOGIN)?;
    serde_json::from_str(&raw).map_err(|_| LOGIN.into())
}

pub(super) fn resolve_gdrive_session_context(
    app: &AppHandle,
) -> Result<(Option<PathBuf>, String, Option<String>), String> {
    let binding = load(app)?;
    let state = app.state::<BrowserState>();
    // Any resident view in the *same* persistent profile has the same cookie
    // manager. Never switch to an active tab in another profile or to main.
    let id = select_profile_view(
        state.snapshot_tabs(),
        &binding.directory,
        |id| crate::window::workspace_data::content_directory(app, id),
        |id| {
            app.get_webview(&crate::state::tab_model::webview_label_for_tab(id))
                .is_some()
        },
    );
    #[cfg(windows)]
    if id.is_none() {
        return Err(LOGIN.into());
    }
    Ok((binding.directory, binding.authuser, id))
}

/// An account index can be reassigned after Google login/logout. Verify the
/// actual identity before every upload/download, and pin the first identity.
pub(super) fn verify_email(app: &AppHandle, email: Option<&str>) -> Result<(), String> {
    let mut binding = load(app)?;
    binding.accept_email(email)?;
    let state = app.state::<BrowserState>();
    let db = state
        .infrastructure()
        .db()
        .lock()
        .map_err(|e| e.to_string())?;
    preferences_repo::store(
        db.as_ref().ok_or(LOGIN)?,
        KEY,
        &serde_json::to_string(&binding).map_err(|_| LOGIN)?,
    )
}

pub(crate) fn clear(conn: &rusqlite::Connection) -> Result<(), String> {
    preferences_repo::remove(conn, KEY)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn profile_selection_never_falls_back_to_another_identity_or_disposable_tab() {
        use crate::state::tab_model::TabInfo;
        let mut tabs: Vec<_> = ["other", "private", "selected"]
            .into_iter()
            .map(|id| {
                TabInfo::new(
                    id.into(),
                    "https://example.test/".into(),
                    "ws-personal".into(),
                    id == "other",
                )
            })
            .collect();
        tabs[1].is_ephemeral = true;
        let chosen = Some(PathBuf::from("selected-profile"));
        let profile = |id: &str| Ok(if id == "other" { None } else { chosen.clone() });
        assert_eq!(
            select_profile_view(tabs.clone(), &chosen, profile, |_| true).as_deref(),
            Some("selected")
        );
        assert!(
            select_profile_view(tabs.clone(), &chosen, profile, |id| id != "selected").is_none()
        );
        assert!(
            select_profile_view(tabs, &chosen, |_| Err("unavailable".into()), |_| true).is_none()
        );
        let mut binding = Binding {
            directory: chosen,
            authuser: "0".into(),
            email: None,
        };
        assert!(binding.accept_email(None).is_err());
        binding.accept_email(Some("fixture@example.test")).unwrap();
        assert!(binding.accept_email(Some("other@example.test")).is_err());
        assert_eq!(binding.email.as_deref(), Some("fixture@example.test"));
        binding.accept_email(Some("FIXTURE@example.test")).unwrap();
    }
    #[test]
    fn account_selection_requires_a_persistent_https_drive_tab() {
        let mut tab = crate::state::tab_model::TabInfo::new(
            "tab-1".into(),
            "https://drive.google.com/drive/u/2/my-drive".into(),
            "ws-personal".into(),
            true,
        );
        assert_eq!(selected_account(&tab).unwrap(), "2");
        tab.is_ephemeral = true;
        assert!(selected_account(&tab).is_err());
        tab.is_ephemeral = false;
        for url in [
            "https://example.test/?next=drive.google.com",
            "http://drive.google.com/drive/u/2/my-drive",
            "https://drive.google.com:444/drive/u/2/my-drive",
            "https://user@drive.google.com/drive/u/2/my-drive",
        ] {
            tab.url = url.into();
            assert!(selected_account(&tab).is_err());
        }
    }
}
