//! GitHub backup commands: a second destination, independent from the sync target in use.
use crate::backup::{
    self, github,
    github_backup::{self, GitHubBackupState, GitHubPushResult, GitHubTarget},
    model::BackupPreferences,
    sync::SyncPullResponse,
};
use tauri::AppHandle;

async fn db<T: Send + 'static>(
    app: &AppHandle,
    work: impl FnOnce(&AppHandle, &rusqlite::Connection) -> Result<T, String> + Send + 'static,
) -> Result<T, String> {
    let worker = app.clone();
    crate::database::executor::run(app.clone(), move |conn| work(&worker, conn)).await
}

#[tauri::command]
pub async fn get_github_backup_state(app: AppHandle) -> Result<GitHubBackupState, String> {
    db(&app, github_backup::state).await
}

/// Checks the token, repository and branch on GitHub, then saves them. An empty token keeps the
/// saved one when the repository does not change.
#[tauri::command]
pub async fn connect_github_backup(
    app: AppHandle,
    repository: String,
    branch: String,
    token: Option<String>,
    auto_save: bool,
) -> Result<GitHubBackupState, String> {
    let repo = github::parse_repository(&repository)?;
    let branch = github::validate_branch(if branch.trim().is_empty() {
        github::DEFAULT_BRANCH
    } else {
        &branch
    })?;
    let lookup = repo.clone();
    let saved = db(&app, move |app, conn| {
        if !backup::encryption::enabled(conn)? {
            return Err("backup.github_encryption_required".to_string());
        }
        github_backup::saved_token_for(app, conn, &lookup)
    })
    .await?;
    let token = match token.filter(|value| !value.trim().is_empty()) {
        Some(value) => github::validate_token(&value)?,
        None => saved.ok_or("backup.github_token_missing")?,
    };
    let (check_repo, check_branch, check_token) = (repo.clone(), branch.clone(), token.clone());
    tauri::async_runtime::spawn_blocking(move || {
        let client = github::Client::new(&check_token, check_repo)?;
        client.check(&check_branch)?;
        if let Some((_, entries)) = client.read_branch(&check_branch)? {
            if github::plan_tree(&entries, "", false, 0).is_none() {
                return Err("backup.github_branch_in_use".to_string());
            }
        }
        Ok(())
    })
    .await
    .map_err(|e| e.to_string())?
    .inspect_err(|error| {
        crate::error_log::warn(&format!(
            "backup: GitHub backup could not connect to {}: {error}",
            repo.full_name()
        ));
    })?;
    let target = GitHubTarget {
        repository: repo.full_name(),
        branch,
        auto_save,
    };
    db(&app, move |app, conn| {
        github_backup::save(app, conn, target, &token)
    })
    .await
}

#[tauri::command]
pub async fn set_github_backup_auto(
    app: AppHandle,
    auto_save: bool,
) -> Result<GitHubBackupState, String> {
    db(&app, move |app, conn| {
        github_backup::set_auto(app, conn, auto_save)
    })
    .await
}

#[tauri::command]
pub async fn disconnect_github_backup(app: AppHandle) -> Result<GitHubBackupState, String> {
    db(&app, github_backup::disconnect).await
}

#[tauri::command]
pub async fn push_github_backup(
    app: AppHandle,
    preferences: BackupPreferences,
    only_if_changed: bool,
) -> Result<GitHubPushResult, String> {
    github_backup::push(&app, preferences, only_if_changed).await
}

#[tauri::command]
pub async fn review_github_backup(
    app: AppHandle,
    current_preferences: BackupPreferences,
) -> Result<SyncPullResponse, String> {
    github_backup::review(&app, current_preferences).await
}

#[tauri::command]
pub async fn list_github_backup_versions(
    app: AppHandle,
) -> Result<Vec<backup::sync_versions::SyncVersion>, String> {
    github_backup::list_versions(&app).await
}

#[tauri::command]
pub async fn review_github_backup_version(
    app: AppHandle,
    id: String,
    current_preferences: BackupPreferences,
) -> Result<SyncPullResponse, String> {
    github_backup::review_version(&app, id, current_preferences).await
}
