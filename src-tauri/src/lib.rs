pub mod commands;
pub mod database;
pub mod privacy;
pub mod state;
pub mod window;

use commands::downloads::{clear_all_downloads, enqueue_download, get_downloads};
use commands::navigation::{go_back, go_forward, navigate_to, reload_page, sync_viewport_bounds};
use commands::persistence::{clear_browsing_history, get_bookmarks, get_history, toggle_bookmark};
use commands::privacy::{get_privacy_stats, toggle_adblock_shield};
use commands::tabs::{
    close_tab, create_tab, get_tabs, hibernate_inactive_tabs, hibernate_tab,
    move_tab_to_workspace, reorder_tabs, set_overlay_visibility, switch_tab,
};
use commands::workspaces::{get_active_workspace, get_workspaces, switch_workspace};
use database::connection::init_database;
use database::session_repo::load_session_tabs;
use state::browser_state::{BrowserState, ViewportBounds, DEFAULT_HOME_URL, INITIAL_TAB_ID};
use tauri::Manager;
use window::vibrancy::apply_native_vibrancy;
use window::webview_manager::spawn_content_webview;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(BrowserState::default())
        .setup(|app| {
            if let Some(main_webview_window) = app.get_webview_window("main") {
                apply_native_vibrancy(&main_webview_window);
            }

            let state = app.state::<BrowserState>();
            let initial_bounds = ViewportBounds::default();

            let mut active_tab_id = INITIAL_TAB_ID.to_string();
            let mut active_tab_url = DEFAULT_HOME_URL.to_string();

            if let Ok(conn) = init_database(app.handle()) {
                if let Ok(restored_tabs) = load_session_tabs(&conn) {
                    if !restored_tabs.is_empty() {
                        if let Some(active) = restored_tabs.iter().find(|t| t.is_active) {
                            active_tab_id = active.id.clone();
                            active_tab_url = active.url.clone();
                            if let Ok(mut ws_guard) = state.active_workspace_id.lock() {
                                *ws_guard = active.workspace_id.clone();
                            }
                        }
                        let max_num = restored_tabs
                            .iter()
                            .filter_map(|t| t.id.strip_prefix("tab-")?.parse::<u64>().ok())
                            .max()
                            .unwrap_or(1);

                        if let Ok(mut counter) = state.next_tab_counter.lock() {
                            *counter = max_num + 1;
                        }
                        if let Ok(mut active_guard) = state.active_tab_id.lock() {
                            *active_guard = active_tab_id.clone();
                        }
                        if let Ok(mut tabs_guard) = state.tabs.lock() {
                            *tabs_guard = restored_tabs;
                        }
                    }
                }
                if let Ok(mut db_guard) = state.db.lock() {
                    *db_guard = Some(conn);
                }
            }

            spawn_content_webview(app.handle(), &active_tab_url, initial_bounds)?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            navigate_to,
            go_back,
            go_forward,
            reload_page,
            sync_viewport_bounds,
            get_tabs,
            create_tab,
            switch_tab,
            close_tab,
            hibernate_tab,
            hibernate_inactive_tabs,
            set_overlay_visibility,
            get_history,
            clear_browsing_history,
            get_bookmarks,
            toggle_bookmark,
            get_privacy_stats,
            toggle_adblock_shield,
            get_workspaces,
            get_active_workspace,
            switch_workspace,
            reorder_tabs,
            move_tab_to_workspace,
            get_downloads,
            enqueue_download,
            clear_all_downloads
        ])
        .run(tauri::generate_context!())
        .expect("error while running Veloce Browser");
}
