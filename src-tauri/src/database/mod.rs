pub mod bookmarks_repo;
pub mod connection;
pub mod downloads_repo;
pub mod executor;
pub mod history_repo;
pub mod migrations;
pub mod navigation_writer;
pub mod new_tab_repo;
pub mod notes_repo;
pub mod saved_sessions_repo;
pub mod session_repo;
pub mod site_icons_repo;
pub mod site_search_repo;
pub mod split_repo;
pub mod tab_usage_repo;
pub mod updates_repo;
pub mod workspace_repo;

pub mod backup_repo;

pub mod closed_tabs_repo;
pub mod account_containers_repo;
pub mod recovery_repo;

pub mod download_jobs_repo;

pub mod site_zoom_repo;

pub mod command_search_repo;
pub mod extensions_repo;
pub mod preferences_repo;
#[cfg(test)]
mod search_benchmarks;
pub mod search_index;
