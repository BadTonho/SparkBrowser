pub mod commands;
pub mod database;
pub mod downloads;
pub mod privacy;
pub mod state;
pub mod window;

use commands::diagnostics::get_diagnostics;
use commands::downloads::{
    cancel_download, clear_all_downloads, enqueue_download, get_download_tasks, get_downloads,
};
use commands::navigation::{go_back, go_forward, navigate_to, reload_page, sync_viewport_bounds};
use commands::notes::{delete_page_note, get_page_note, save_page_note, search_page_notes};
use commands::overlays::{
    set_bookmark_context_menu_overlay, set_overlay_visibility, set_workspace_switcher_overlay,
};
use commands::persistence::{
    clear_browsing_history, create_bookmark_folder, delete_bookmark_folder, get_bookmark_data,
    get_history, remove_bookmark, rename_bookmark_folder, save_bookmark,
};
use commands::privacy::{get_privacy_stats, toggle_adblock_shield};
use commands::saved_sessions::{
    delete_named_session, get_saved_sessions, rename_named_session, restore_named_session,
    save_named_session,
};
use commands::shortcuts::set_keyboard_shortcuts;
use commands::site_search::{
    delete_site_search_shortcut, get_site_search_shortcuts, save_site_search_shortcut,
};
use commands::tab_organizer::close_tabs_reviewed;
use commands::tabs::{
    auto_hibernate_inactive_tabs, close_tab, create_tab, get_tabs, hibernate_inactive_tabs,
    hibernate_tab, move_tab_to_workspace, reorder_tabs, set_resident_limit, set_tab_pinned,
    switch_tab,
};
use commands::workspaces::{
    create_workspace, delete_workspace, get_active_workspace, get_workspaces, switch_workspace,
    update_workspace,
};
use database::connection::init_database;
use database::session_repo::load_session_tabs;
use database::workspace_repo::list_workspaces;
use state::browser_state::{BrowserState, INITIAL_TAB_ID};
use tauri::{Emitter, Manager, WindowEvent};
use window::vibrancy::apply_native_vibrancy;
use window::webview_manager::{spawn_content_webview, BOOKMARK_CONTEXT_MENU_WEBVIEW_LABEL};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(BrowserState::default())
        .manage(downloads::manager::DownloadManager::default())
        .on_window_event(|window, event| {
            if window.label() == "main" && matches!(event, WindowEvent::Focused(false)) {
                if let Some(state) = window.try_state::<BrowserState>() {
                    state
                        .workspace_alt_pressed
                        .store(false, std::sync::atomic::Ordering::SeqCst);
                    if let Ok(mut token) = state.bookmark_context_menu_token.lock() {
                        *token = None;
                    }
                    if let Ok(mut actions) = state.bookmark_context_menu_actions.lock() {
                        actions.clear();
                    }
                    if let Ok(mut open) = state.bookmark_context_menu_open.lock() {
                        *open = false;
                    }
                    if let Some(webview) = window
                        .app_handle()
                        .get_webview(BOOKMARK_CONTEXT_MENU_WEBVIEW_LABEL)
                    {
                        let _ = webview.hide();
                    }
                }
                let _ = window.emit("workspace-shortcut-cancel", ());
            }
        })
        .setup(|app| {
            if let Some(main_webview_window) = app.get_webview_window("main") {
                apply_native_vibrancy(&main_webview_window);
            }

            let state = app.state::<BrowserState>();

            let mut active_tab_id = INITIAL_TAB_ID.to_string();

            {
                let conn = init_database(app.handle()).map_err(std::io::Error::other)?;
                if let Ok(workspaces) = list_workspaces(&conn) {
                    if !workspaces.is_empty() {
                        if let Ok(mut workspace_guard) = state.workspaces.lock() {
                            *workspace_guard = workspaces.clone();
                        }
                    }
                }
                {
                    let mut restored_tabs =
                        load_session_tabs(&conn).map_err(std::io::Error::other)?;
                    let valid_ids: Vec<String> = state
                        .workspaces
                        .lock()
                        .map(|workspaces| workspaces.iter().map(|ws| ws.id.clone()).collect())
                        .unwrap_or_default();
                    for tab in &mut restored_tabs {
                        if !valid_ids.iter().any(|id| id == &tab.workspace_id) {
                            tab.workspace_id = "ws-personal".to_string();
                        }
                    }
                    if !restored_tabs.is_empty() {
                        if let Some(active) = restored_tabs.iter().find(|t| t.is_active) {
                            active_tab_id = active.id.clone();
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

            app.manage(database::navigation_writer::NavigationWriter::start(
                app.handle().clone(),
            )?);
            spawn_content_webview(app.handle())?;

            Ok(())
        })
        .invoke_handler(commands::access::shell_only(tauri::generate_handler![
            get_saved_sessions,
            save_named_session,
            rename_named_session,
            delete_named_session,
            restore_named_session,
            get_site_search_shortcuts,
            save_site_search_shortcut,
            delete_site_search_shortcut,
            get_page_note,
            search_page_notes,
            save_page_note,
            delete_page_note,
            close_tabs_reviewed,
            get_diagnostics,
            navigate_to,
            go_back,
            go_forward,
            reload_page,
            sync_viewport_bounds,
            get_tabs,
            set_resident_limit,
            set_tab_pinned,
            create_tab,
            switch_tab,
            close_tab,
            hibernate_tab,
            hibernate_inactive_tabs,
            auto_hibernate_inactive_tabs,
            set_overlay_visibility,
            set_workspace_switcher_overlay,
            set_bookmark_context_menu_overlay,
            set_keyboard_shortcuts,
            get_history,
            clear_browsing_history,
            get_bookmark_data,
            save_bookmark,
            remove_bookmark,
            create_bookmark_folder,
            rename_bookmark_folder,
            delete_bookmark_folder,
            get_privacy_stats,
            toggle_adblock_shield,
            get_workspaces,
            get_active_workspace,
            switch_workspace,
            create_workspace,
            update_workspace,
            delete_workspace,
            reorder_tabs,
            move_tab_to_workspace,
            get_downloads,
            get_download_tasks,
            cancel_download,
            enqueue_download,
            clear_all_downloads
        ]))
        .build(tauri::generate_context!())
        .expect("error while building Veloce Browser")
        .run(|app, event| {
            if let tauri::RunEvent::ExitRequested {
                code: None, api, ..
            } = event
            {
                api.prevent_exit();
                let app = app.clone();
                std::thread::spawn(move || {
                    app.state::<downloads::manager::DownloadManager>()
                        .shutdown();
                    if app
                        .state::<database::navigation_writer::NavigationWriter>()
                        .flush()
                        .is_err()
                    {
                        eprintln!("Navigation persistence failed before shutdown");
                    }
                    app.exit(0);
                });
            }
        });
}
