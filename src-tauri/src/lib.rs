use commands::backup::{
    choose_backup_file, commit_backup_import, discard_backup_import, export_backup,
    finish_backup_preferences, get_backup_pending_preferences, preview_backup_import,
};
use commands::closed_tabs::{clear_closed_tabs, get_closed_tabs, reopen_closed_tab};
use commands::command_search::{get_saved_session, search_command_items};
use commands::new_tab::set_new_tab_preferences;
use commands::shutdown::{cancel_shutdown, confirm_shutdown, get_shutdown_state, prepare_shutdown};
use commands::webview_runtime::{get_webview_runtime_status, restart_webview_runtime};
use commands::zoom::{apply_backup_zoom, get_site_zoom, set_site_zoom};
pub mod backup;
pub mod commands;
pub mod database;
pub mod downloads;
pub mod media;
mod native_tests;
pub mod privacy;
pub mod recovery;
pub mod security;
pub mod shutdown;
pub mod site_permissions;
pub mod state;
pub mod updates;
pub mod webview_runtime;
pub mod window;
pub mod zoom;
use commands::media::{
    enter_video_pip, exit_video_pip, get_tab_media, refresh_tab_media, set_pip_window_shadow,
    set_tab_muted, set_tab_volume,
};

use commands::diagnostics::get_diagnostics;
use commands::downloads::{
    cancel_download, clear_all_downloads, control_download, enqueue_download, get_download_tasks,
    get_download_visual, get_downloads, set_download_checksum,
};
use commands::navigation::{
    focus_address_bar, go_back, go_forward, navigate_to, reload_page, sync_viewport_bounds,
};
use commands::notes::{delete_page_note, get_page_note, save_page_note, search_page_notes};
use commands::overlays::{
    set_bookmark_context_menu_overlay, set_downloads_popover, set_media_popover,
    set_overlay_visibility, set_privacy_popover, set_tools_popover, set_workspace_switcher_overlay,
    set_zoom_popover,
};
use commands::persistence::{
    clear_browsing_history, create_bookmark_folder, delete_bookmark_folder, get_bookmark_data,
    get_history, remove_bookmark, rename_bookmark_folder, save_bookmark,
};
use commands::privacy::{get_privacy_stats, set_auto_skip_ads, toggle_adblock_shield};
use commands::recovery::get_recovery_status;
use commands::saved_sessions::{
    delete_named_session, get_saved_sessions, rename_named_session, restore_named_session,
    save_named_session,
};
use commands::shortcuts::set_keyboard_shortcuts;
use commands::site_permissions::{
    get_site_permissions, respond_site_permission, set_site_permission,
};
use commands::site_search::{
    delete_site_search_shortcut, get_site_search_shortcuts, save_site_search_shortcut,
};
use commands::split_view::{
    commit_split_ratio, dock_tab_to_split, move_tab_in_split, set_split_view,
};
use commands::tab_organizer::close_tabs_reviewed;
use commands::tabs::{
    auto_hibernate_inactive_tabs, close_tab, create_tab, get_tabs, hibernate_inactive_tabs,
    hibernate_tab, move_tab_to_workspace, reorder_tabs, set_resident_limit, set_tab_pinned,
    switch_tab,
};
use commands::updates::{
    check_for_updates, download_update, get_update_state, install_update, set_update_channel,
};
use commands::workspaces::{
    create_workspace, delete_workspace, get_active_workspace, get_workspace_isolation,
    get_workspaces, set_workspace_isolation, switch_workspace, update_workspace,
};
use database::connection::init_database;
use state::browser_state::BrowserState;
use tauri::{Manager, WindowEvent};
use window::tab_drag_preview::set_tab_drag_preview;
use window::vibrancy::apply_native_vibrancy;
use window::webview_manager::{spawn_content_webview, BOOKMARK_CONTEXT_MENU_WEBVIEW_LABEL};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    webview_runtime::restart::wait_for_parent()
        .expect("Runtime restart could not wait for the previous process");
    updates::initialize_crypto();
    let mut context = tauri::generate_context!();
    native_tests::configure(&mut context).expect("invalid native test directory");
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(BrowserState::default())
        .manage(webview_runtime::RuntimeService::default())
        .manage(window::new_tab::NewTabService::default())
        .manage(window::new_tab::widgets::Widgets::default())
        .manage(window::tab_drag_preview::DragPreview::default())
        .manage(zoom::ZoomService::default())
        .manage(media::MediaService::default())
        .manage(window::media_popover::MediaPopover::default())
        .manage(window::tools_popover::ToolsPopover::default())
        .manage(window::downloads_popover::DownloadsPopover::default())
        .manage(window::privacy_popover::PrivacyPopover::default())
        .manage(window::zoom_popover::ZoomPopover::default())
        .manage(shutdown::ShutdownService::default())
        .manage(backup::service::BackupService::default())
        .manage(site_permissions::PermissionRequests::default())
        .manage(downloads::manager::DownloadManager::default())
        .on_window_event(|window, event| {
            if window.label() == "main" {
                if let WindowEvent::CloseRequested { api, .. } = event {
                    api.prevent_close();
                    if let Some(service) = window.try_state::<shutdown::ShutdownService>() {
                        let _ = service.request(window.app_handle(), None);
                    }
                    return;
                }
            }
            if window.label() == "main" && matches!(event, WindowEvent::Focused(false)) {
                window::zoom_popover::focus_lost(window.app_handle());
                window::tools_popover::focus_lost(window.app_handle());
                window::downloads_popover::focus_lost(window.app_handle());
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
                let _ = crate::security::emit_shell(
                    window.app_handle(),
                    "workspace-shortcut-cancel",
                    (),
                );
            }
        })
        .setup(|app| {
            #[cfg(windows)]
            app.handle()
                .plugin(tauri_plugin_updater::Builder::new().build())?;
            if let Some(main_webview_window) = app.get_webview_window("main") {
                apply_native_vibrancy(&main_webview_window);
            }

            if let Some(view) = app.get_webview("main") {
                security::harden(app.handle(), &view, true).map_err(std::io::Error::other)?;
                webview_runtime::initialize(app.handle(), &view).map_err(std::io::Error::other)?;
            }
            let state = app.state::<BrowserState>();

            let data_directory = app.path().app_data_dir()?;
            app.manage(
                recovery::profile_lock::ProfileLock::acquire(&data_directory)
                    .map_err(std::io::Error::other)?,
            );
            let conn = init_database(app.handle()).map_err(std::io::Error::other)?;
            database::preferences_repo::initialize_preferences_table(&conn)
                .map_err(std::io::Error::other)?;
            state
                .workspace_isolation
                .store(
                    database::preferences_repo::load_bool(
                        &conn,
                        database::preferences_repo::WORKSPACE_ISOLATION_KEY,
                    )
                    .unwrap_or(false),
                    std::sync::atomic::Ordering::Release,
                );
            let recovery_status =
                recovery::startup::restore(&state, conn).map_err(std::io::Error::other)?;
            app.manage(recovery::RecoveryService::new(recovery_status));
            app.state::<downloads::manager::DownloadManager>()
                .initialize(app.handle())
                .map_err(std::io::Error::other)?;

            app.manage(database::navigation_writer::NavigationWriter::start(
                app.handle().clone(),
            )?);
            app.manage(updates::UpdateService::new(app.handle()).map_err(std::io::Error::other)?);
            app.state::<zoom::ZoomService>()
                .initialize(app.handle())
                .map_err(std::io::Error::other)?;
            window::new_tab::widgets::initialize(app.handle());
            spawn_content_webview(app.handle())?;
            state
                .startup_ready
                .store(true, std::sync::atomic::Ordering::Release);

            Ok(())
        })
        .invoke_handler(commands::access::shell_only(tauri::generate_handler![
            get_webview_runtime_status,
            restart_webview_runtime,
            focus_address_bar,
            set_new_tab_preferences,
            refresh_tab_media,
            set_tab_volume,
            enter_video_pip,
            set_pip_window_shadow,
            exit_video_pip,
            get_tab_media,
            set_tab_muted,
            search_command_items,
            get_saved_session,
            get_site_zoom,
            set_site_zoom,
            apply_backup_zoom,
            get_shutdown_state,
            prepare_shutdown,
            confirm_shutdown,
            cancel_shutdown,
            control_download,
            set_download_checksum,
            get_recovery_status,
            get_closed_tabs,
            reopen_closed_tab,
            clear_closed_tabs,
            export_backup,
            choose_backup_file,
            discard_backup_import,
            preview_backup_import,
            commit_backup_import,
            get_backup_pending_preferences,
            finish_backup_preferences,
            get_update_state,
            set_update_channel,
            check_for_updates,
            download_update,
            install_update,
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
            set_split_view,
            dock_tab_to_split,
            move_tab_in_split,
            set_tab_drag_preview,
            commit_split_ratio,
            get_site_permissions,
            set_site_permission,
            respond_site_permission,
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
            set_media_popover,
            set_zoom_popover,
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
            set_auto_skip_ads,
            set_tools_popover,
            set_downloads_popover,
            set_privacy_popover,
            get_workspaces,
            get_active_workspace,
            switch_workspace,
            get_workspace_isolation,
            set_workspace_isolation,
            create_workspace,
            update_workspace,
            delete_workspace,
            reorder_tabs,
            move_tab_to_workspace,
            get_downloads,
            get_download_tasks,
            get_download_visual,
            cancel_download,
            enqueue_download,
            clear_all_downloads
        ]))
        .build(context)
        .expect("error while building Veloce Browser")
        .run(|app, event| {
            if let tauri::RunEvent::ExitRequested {
                code: None, api, ..
            } = event
            {
                api.prevent_exit();
                let _ = app.state::<shutdown::ShutdownService>().request(app, None);
            }
        });
}
