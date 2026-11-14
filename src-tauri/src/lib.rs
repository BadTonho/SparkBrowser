use commands::account_containers::{
    create_account_container, create_container_tab, delete_account_container,
    get_account_containers, rename_account_container, set_tab_account_container,
};
use commands::backup::{
    cancel_backup_unlock, choose_backup_file, choose_backup_sync_folder, commit_backup_import,
    create_local_backup, delete_local_backup, discard_backup_import, export_backup,
    finish_backup_preferences, get_backup_pending_preferences, get_backup_sync_state,
    get_local_backup_state, preview_backup_import, pull_backup_sync, push_backup_sync,
    restore_local_backup, set_backup_encryption, set_backup_sync_config, set_local_backup_schedule,
    unlock_backup_file,
};
use commands::bookmark_export::export_bookmarks;
use commands::capture_probe::full_page_capture_probe;
use commands::closed_tabs::{clear_closed_tabs, get_closed_tabs, reopen_closed_tab};
use commands::command_search::{get_saved_session, search_command_items};
use commands::dark_mode_probe::set_dark_mode_probe_scheme;
use commands::graphics::{get_graphics_status, restart_for_graphics, set_hardware_acceleration};
use commands::new_tab::set_new_tab_preferences;
use commands::private_session::open_private_window;
use commands::shutdown::{cancel_shutdown, confirm_shutdown, get_shutdown_state, prepare_shutdown};
use commands::webview_runtime::{get_webview_runtime_status, restart_webview_runtime};
use commands::zoom::{apply_backup_zoom, get_site_zoom, set_site_zoom};
pub mod backup;
pub mod commands;
pub mod database;
pub mod downloads;
pub mod extensions;
pub mod feeds;
pub mod graphics;
pub mod legacy;
pub mod media;
mod native_tests;
pub mod privacy;
pub mod private_session;
pub mod recovery;
pub mod search_engines;
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
use commands::minimum_font_probe::set_minimum_font_probe_size;

use commands::diagnostics::get_diagnostics;
use commands::downloads::{
    cancel_download, clear_all_downloads, control_download, enqueue_download, get_download_tasks,
    get_download_visual, get_downloads, set_download_checksum,
};
use commands::experiments::{
    get_experimental_flags, reset_experimental_flags, set_experimental_flag,
};
use commands::extensions::{
    get_extension_action, get_installed_extensions, install_extension_from_archive,
    install_extension_from_url, load_unpacked_extension, remove_extension, toggle_extension,
};
use commands::feeds::{delete_feed, get_feed_entries, get_feeds, refresh_feed, save_feed};
use commands::navigation::{
    focus_address_bar, go_back, go_forward, navigate_to, reload_page, sync_viewport_bounds,
};
use commands::notes::{delete_page_note, get_page_note, save_page_note, search_page_notes};
use commands::overlays::{
    set_bookmark_context_menu_overlay, set_downloads_popover, set_history_overlay,
    set_media_popover, set_overlay_visibility, set_permission_popover, set_privacy_popover,
    set_quick_settings_popover, set_settings_overlay, set_tools_popover,
    set_workspace_switcher_overlay, set_zoom_popover,
};
use commands::persistence::{
    clear_browsing_history, create_bookmark_folder, delete_bookmark_folder, get_bookmark_data,
    get_history, remove_bookmark, rename_bookmark_folder, save_bookmark,
};
use commands::privacy::{
    get_gpc, get_privacy_stats, get_shield_exceptions, get_tracking_cleaning, set_auto_skip_ads,
    set_gpc, set_shield_exceptions, set_tracking_cleaning, toggle_adblock_shield,
};
use commands::privacy_settings::{
    get_privacy_settings, set_history_retention, set_popup_setting, set_smart_screen,
};
use commands::qr_code::save_qr_code_png;
use commands::reader::get_reader_article;
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
    auto_hibernate_inactive_tabs, close_tab, create_ephemeral_tab, create_tab, get_tabs,
    get_tabs_memory, hibernate_inactive_tabs, hibernate_tab, move_tab_to_workspace, reorder_tabs,
    set_ask_download_destination, set_hibernation_exemptions, set_resident_limit, set_tab_pinned,
    switch_tab,
};
use commands::updates::{
    check_for_updates, download_update, get_update_state, install_update, set_update_channel,
};
use commands::whatsapp::toggle_whatsapp_popover;
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
    let mode = private_session::Mode::from_args().expect("invalid private session arguments");
    if let private_session::Mode::Cleanup(root) = &mode {
        let _ = private_session::remove_session(root);
        return;
    }
    webview_runtime::restart::wait_for_parent()
        .expect("Runtime restart could not wait for the previous process");
    updates::initialize_crypto();
    let mut context = tauri::generate_context!();
    native_tests::configure(&mut context).expect("invalid native test directory");
    // Move a profile created under the old name before any webview opens it.
    if matches!(mode, private_session::Mode::Normal) {
        legacy::migrate_profile(&context.config().identifier);
    }
    // Graphics flags are read once, when the first webview starts, so they are set here. Private
    // windows and restarted children keep whatever their parent put in the environment.
    if matches!(mode, private_session::Mode::Normal) {
        let accelerated = graphics::data_directory(context.config())
            .is_none_or(|directory| graphics::load(&directory));
        graphics::apply_environment(accelerated);
    }
    let graphics_service = graphics::GraphicsService::new(graphics::running_accelerated());
    let role = private_session::PrivateRole::from_mode(mode);
    private_session::configure(&mut context, role.launch());
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(role)
        .manage(graphics_service)
        .manage(BrowserState::default())
        .manage(window::fullscreen::FullscreenState::default())
        .manage(webview_runtime::RuntimeService::default())
        .manage(window::new_tab::NewTabService::default())
        .manage(window::new_tab::widgets::Widgets::default())
        .manage(window::tab_drag_preview::DragPreview::default())
        .manage(zoom::ZoomService::default())
        .manage(media::MediaService::default())
        .manage(window::media_popover::MediaPopover::default())
        .manage(window::tools_popover::ToolsPopover::default())
        .manage(window::downloads_popover::DownloadsPopover::default())
        .manage(window::quick_settings_popover::QuickSettingsPopover::default())
        .manage(window::settings_overlay::SettingsOverlay::default())
        .manage(window::privacy_popover::PrivacyPopover::default())
        .manage(window::permission_popover::PermissionPopover::default())
        .manage(window::history_overlay::HistoryOverlay::default())
        .manage(window::zoom_popover::ZoomPopover::default())
        .manage(shutdown::ShutdownService::default())
        .manage(backup::service::BackupService::default())
        .manage(site_permissions::PermissionRequests::default())
        .manage(downloads::manager::DownloadManager::default())
        .on_window_event(|window, event| {
            if window.label() == "main"
                && matches!(event, WindowEvent::Resized(_))
                && window::fullscreen::active(window.app_handle()).is_some()
            {
                let app = window.app_handle();
                if let Some(state) = app.try_state::<BrowserState>() {
                    let _ = window::viewport::sync_content_geometry(app, &state);
                }
            }
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
                window::quick_settings_popover::focus_lost(window.app_handle());
                if let Some(state) = window.try_state::<BrowserState>() {
                    state
                        .surfaces()
                        .workspace_alt_pressed()
                        .store(false, std::sync::atomic::Ordering::SeqCst);
                    if let Ok(mut token) = state.surfaces().bookmark_context_menu_token().lock() {
                        *token = None;
                    }
                    if let Ok(mut actions) = state.surfaces().bookmark_context_menu_actions().lock()
                    {
                        actions.clear();
                    }
                    if let Ok(mut open) = state.surfaces().bookmark_context_menu_open().lock() {
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
                #[cfg(target_os = "linux")]
                window::linux_layout::setup(&main_webview_window).map_err(std::io::Error::other)?;
            }

            if let Some(view) = app.get_webview("main") {
                security::harden(app.handle(), &view, true).map_err(std::io::Error::other)?;
                webview_runtime::initialize(app.handle(), &view).map_err(std::io::Error::other)?;
            }
            let state = app.state::<BrowserState>();

            let data_directory = app.path().app_data_dir()?;
            let lists = privacy::filter_lists::FilterLists::start(
                &data_directory.join("filter-lists").join("last-valid.json"),
                !cfg!(test) && app.config().identifier != "org.spark.browser.native-tests",
            );
            app.manage(lists);
            app.manage(
                recovery::profile_lock::ProfileLock::acquire(&data_directory)
                    .map_err(std::io::Error::other)?,
            );
            window::ephemeral::purge_all_ephemeral_data(app.handle())
                .map_err(std::io::Error::other)?;
            if app.state::<private_session::PrivateRole>().is_private() {
                if let Some(window) = app.get_webview_window("main") {
                    window
                        .set_title("Spark - Privada")
                        .map_err(std::io::Error::other)?;
                }
                private_session::seed_profile(app.handle()).map_err(std::io::Error::other)?;
            } else {
                private_session::purge_stale_sessions(app.handle())
                    .map_err(std::io::Error::other)?;
            }
            let conn = init_database(app.handle()).map_err(std::io::Error::other)?;
            database::preferences_repo::initialize_preferences_table(&conn)
                .map_err(std::io::Error::other)?;
            // Secrets saved under the pre-rename names follow the profile; a locked keyring just retries next start.
            let _ = backup::migrate_legacy_secrets(
                &conn,
                &app.state::<backup::service::BackupService>()
                    .sync
                    .credentials,
            );
            let legacy_credential = database::preferences_repo::load(
                &conn,
                database::preferences_repo::BACKUP_SYNC_SECRET_KEY,
            )
            .map_err(std::io::Error::other)?
            .is_some();
            if legacy_credential
                && backup::sync::resolve_secret(
                    &conn,
                    &app.state::<backup::service::BackupService>().sync,
                )
                .is_err()
            {
                let service = app.state::<backup::service::BackupService>();
                *service
                    .sync
                    .last_error
                    .lock()
                    .map_err(|_| std::io::Error::other("Sync state unavailable"))? =
                    Some("backup.sync_secure_storage_failed".into());
            }
            if let Some(settings) = database::preferences_repo::load(
                &conn,
                database::preferences_repo::TRACKING_CLEANING_KEY,
            )
            .ok()
            .flatten()
            .and_then(|value| {
                serde_json::from_str::<privacy::tracking_parameters::TrackingSettings>(&value).ok()
            })
            .and_then(|settings| settings.validate().ok())
            {
                state
                    .policies()
                    .set_tracking_settings(settings)
                    .map_err(|_| std::io::Error::other("Tracking settings unavailable"))?;
            }
            if let Some(settings) =
                database::preferences_repo::load(&conn, database::preferences_repo::GPC_KEY)
                    .ok()
                    .flatten()
                    .and_then(|value| {
                        serde_json::from_str::<privacy::tracking_parameters::TrackingSettings>(
                            &value,
                        )
                        .ok()
                    })
                    .and_then(|settings| settings.validate().ok())
            {
                state
                    .policies()
                    .set_gpc_settings(settings)
                    .map_err(|_| std::io::Error::other("GPC settings unavailable"))?;
            }
            if let Some(settings) = database::preferences_repo::load(
                &conn,
                database::preferences_repo::SHIELD_EXCEPTIONS_KEY,
            )
            .ok()
            .flatten()
            .and_then(|value| {
                serde_json::from_str::<privacy::tracking_parameters::TrackingSettings>(&value).ok()
            })
            .and_then(|settings| settings.validate().ok())
            {
                state
                    .policies()
                    .set_shield_settings(settings)
                    .map_err(|_| std::io::Error::other("Shield settings unavailable"))?;
            }
            state.policies().set_workspace_isolation(
                database::preferences_repo::load_bool(
                    &conn,
                    database::preferences_repo::WORKSPACE_ISOLATION_KEY,
                )
                .unwrap_or(false),
            );
            let history_retention =
                crate::database::history_repo::HistoryRetention::from_preference(
                    crate::database::preferences_repo::load(
                        &conn,
                        crate::database::preferences_repo::HISTORY_RETENTION_KEY,
                    )
                    .ok()
                    .flatten(),
                );
            state
                .policies()
                .set_history_retention_months(history_retention.state_value());
            state.policies().set_allow_popups(
                crate::database::preferences_repo::load_bool(
                    &conn,
                    crate::database::preferences_repo::ALLOW_POPUPS_KEY,
                )
                .unwrap_or(false),
            );
            state.policies().set_smart_screen_enabled(
                crate::database::preferences_repo::load(
                    &conn,
                    crate::database::preferences_repo::SMART_SCREEN_KEY,
                )
                .ok()
                .flatten()
                .as_deref()
                    != Some("false"),
            );
            crate::database::history_repo::prune_expired(
                &conn,
                history_retention,
                crate::database::history_repo::current_unix_timestamp(),
            )
            .map_err(std::io::Error::other)?;
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
                .infrastructure()
                .startup_ready()
                .store(true, std::sync::atomic::Ordering::Release);
            feeds::start(app.handle().clone());

            Ok(())
        })
        .invoke_handler(commands::access::shell_only(tauri::generate_handler![
            get_feeds,
            get_experimental_flags,
            set_experimental_flag,
            reset_experimental_flags,
            get_feed_entries,
            save_feed,
            delete_feed,
            refresh_feed,
            get_tracking_cleaning,
            set_tracking_cleaning,
            get_gpc,
            set_gpc,
            get_shield_exceptions,
            set_shield_exceptions,
            get_account_containers,
            create_account_container,
            rename_account_container,
            delete_account_container,
            set_tab_account_container,
            create_container_tab,
            full_page_capture_probe,
            set_dark_mode_probe_scheme,
            set_minimum_font_probe_size,
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
            get_graphics_status,
            set_hardware_acceleration,
            restart_for_graphics,
            export_bookmarks,
            choose_backup_file,
            discard_backup_import,
            preview_backup_import,
            commit_backup_import,
            get_backup_pending_preferences,
            finish_backup_preferences,
            get_backup_sync_state,
            choose_backup_sync_folder,
            set_backup_sync_config,
            push_backup_sync,
            pull_backup_sync,
            unlock_backup_file,
            cancel_backup_unlock,
            get_local_backup_state,
            set_local_backup_schedule,
            set_backup_encryption,
            create_local_backup,
            delete_local_backup,
            restore_local_backup,
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
            get_tabs_memory,
            set_resident_limit,
            set_hibernation_exemptions,
            set_ask_download_destination,
            set_tab_pinned,
            create_tab,
            create_ephemeral_tab,
            open_private_window,
            get_reader_article,
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
            get_privacy_settings,
            set_history_retention,
            set_popup_setting,
            set_smart_screen,
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
            set_quick_settings_popover,
            set_settings_overlay,
            set_privacy_popover,
            set_permission_popover,
            set_history_overlay,
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
            clear_all_downloads,
            get_installed_extensions,
            load_unpacked_extension,
            toggle_extension,
            remove_extension,
            get_extension_action,
            install_extension_from_url,
            install_extension_from_archive,
            save_qr_code_png,
            toggle_whatsapp_popover
        ]))
        .build(context)
        .expect("error while building Spark")
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
