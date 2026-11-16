use crate::database::tab_usage_repo;
use crate::state::browser_state::{BrowserState, TabSnapshot, DEFAULT_HOME_URL};
use crate::state::workspace_model::DEFAULT_WORKSPACE_ID;
use crate::state::{tab_checkpoint::TabCheckpoint, tab_model::TabInfo};
use tauri::{AppHandle, Manager, State};

pub fn popup_target_allowed(enabled: bool, url: &tauri::Url) -> bool {
    enabled
        && matches!(url.scheme(), "http" | "https")
        && url.host_str().is_some()
        && url.username().is_empty()
        && url.password().is_none()
}

/// Opens an authenticated user-selected link without activating the new tab.
pub fn open_user_link(app: AppHandle, source_tab: String, generation: String, url: tauri::Url) {
    if !popup_target_allowed(true, &url) {
        return;
    }
    tauri::async_runtime::spawn(async move {
        let state = app.state::<BrowserState>();
        let _operation = state.session().operation().await;
        let label = crate::state::tab_model::webview_label_for_tab(&source_tab);
        if !crate::window::navigation_state::is_current_generation(&app, &label, &generation) {
            return;
        }
        let source = state
            .session()
            .edit_tabs()
            .lock()
            .ok()
            .and_then(|tabs| tabs.iter().find(|tab| tab.id == source_tab).cloned());
        let Some(source) = source else {
            return;
        };
        let shield = state
            .policies()
            .shield_enabled()
            .lock()
            .map(|v| *v)
            .unwrap_or(true);
        if crate::privacy::adblock_engine::site_is_protected(&state, url.as_str())
            && crate::privacy::adblock_engine::should_block_url(url.as_str(), shield)
        {
            return;
        }
        let id = match state.session().edit_next_tab_counter().lock() {
            Ok(mut next) => {
                let id = format!("tab-{}", *next);
                *next += 1;
                id
            }
            Err(_) => return,
        };
        let result = TabCheckpoint::commit(&state, || {
            let mut tab = if source.is_ephemeral {
                TabInfo::new_ephemeral(
                    id.clone(),
                    url.to_string(),
                    source.workspace_id.clone(),
                    false,
                )
            } else {
                TabInfo::new(
                    id.clone(),
                    url.to_string(),
                    source.workspace_id.clone(),
                    false,
                )
            };
            if !source.is_ephemeral {
                tab.container_id = source.container_id.clone();
            }
            tab.is_hibernated = true;
            tab.window_id.clone_from(&source.window_id);
            state
                .session()
                .edit_tabs()
                .lock()
                .map_err(|e| e.to_string())?
                .push(tab);
            Ok(())
        });
        if result.is_ok() {
            reconcile_committed(&app, &state);
            crate::state::publish::publish_tabs(&app, &state, &source.window_id);
        }
    });
}

/// Web pop-ups open as normal browser tabs; internal PiP windows are handled separately.
pub fn open_popup_url(app: AppHandle, source_tab: String, generation: String, url: tauri::Url) {
    if !popup_target_allowed(app.state::<BrowserState>().policies().allow_popups(), &url) {
        return;
    }
    tauri::async_runtime::spawn(async move {
        let state = app.state::<BrowserState>();
        let _operation = state.session().operation().await;
        let label = crate::state::tab_model::webview_label_for_tab(&source_tab);
        if !crate::window::navigation_state::is_current_generation(&app, &label, &generation) {
            return;
        }
        if !popup_target_allowed(state.policies().allow_popups(), &url) {
            return;
        }
        let source = state
            .session()
            .edit_tabs()
            .lock()
            .ok()
            .and_then(|tabs| tabs.iter().find(|tab| tab.id == source_tab).cloned());
        if let Some(source) = source {
            crate::error_log::ignored(create_tab_inner_ext(
                &app,
                &state,
                Some(url.to_string()),
                None,
                None,
                TabProfile {
                    is_ephemeral: source.is_ephemeral,
                    container_id: source.container_id,
                    workspace_id: Some(source.workspace_id),
                    window_id: Some(source.window_id),
                },
            ));
        }
    });
}

#[tauri::command]
pub async fn get_tabs(
    webview: tauri::Webview,
    state: State<'_, BrowserState>,
) -> Result<TabSnapshot, String> {
    let _operation = state.session().operation().await;
    Ok(state.snapshot_packet_for(&crate::commands::access::caller_window(&webview)))
}

#[tauri::command]
pub async fn get_tabs_memory(
    app: AppHandle,
    state: State<'_, BrowserState>,
) -> Result<std::collections::HashMap<String, u64>, String> {
    let _operation = state.session().operation().await;
    let tabs = state
        .session()
        .edit_tabs()
        .lock()
        .map_err(|e| e.to_string())?
        .clone();
    let resident_tabs: Vec<_> = tabs.iter().filter(|t| !t.is_hibernated).collect();
    let mut result = std::collections::HashMap::new();
    for tab in &tabs {
        if tab.is_hibernated {
            result.insert(tab.id.clone(), 0);
        }
    }

    let isolated_workspaces = state.policies().workspace_isolation();
    let candidates: Vec<_> = resident_tabs
        .iter()
        .filter_map(|tab| {
            let label = crate::state::tab_model::webview_label_for_tab(&tab.id);
            let view = app.get_webview(&label)?;
            let source = crate::window::tab_memory::current_webview_source(&view)?;
            let environment_group = if isolated_workspaces {
                tab.workspace_id.clone()
            } else {
                String::new()
            };
            Some((tab.id.clone(), view, source, environment_group))
        })
        .collect();
    let mut resident_sources_by_environment =
        std::collections::HashMap::<String, Vec<String>>::new();
    for (_, _, source, environment_group) in &candidates {
        resident_sources_by_environment
            .entry(environment_group.clone())
            .or_default()
            .push(source.clone());
    }
    let mut measurements = tokio::task::JoinSet::new();
    for (tab_id, view, source, environment_group) in candidates {
        let sources = resident_sources_by_environment
            .get(&environment_group)
            .cloned()
            .unwrap_or_default();
        measurements.spawn(async move {
            let bytes = crate::window::tab_memory::measure_webview_memory(&view, source, sources)
                .await
                .unwrap_or(0);
            (tab_id, bytes)
        });
    }
    while let Some(measurement) = measurements.join_next().await {
        if let Ok((tab_id, bytes)) = measurement {
            result.insert(tab_id, bytes);
        }
    }
    for tab in resident_tabs {
        result.entry(tab.id.clone()).or_insert(0);
    }
    Ok(result)
}

#[tauri::command]
pub async fn create_tab(
    app: AppHandle,
    webview: tauri::Webview,
    state: State<'_, BrowserState>,
    url: Option<String>,
    search_engine: Option<String>,
    side: Option<crate::state::split_docking::DockSide>,
) -> Result<TabSnapshot, String> {
    let mut span = state.infrastructure().diagnostics().span("tabs.create");
    let _operation = state.session().operation().await;
    let window_id = crate::commands::access::caller_window(&webview);
    let result = create_tab_inner(&app, &state, url, search_engine, side, &window_id);
    span.finish(result.is_ok(), 0);
    result
}

#[tauri::command]
pub async fn create_ephemeral_tab(
    app: AppHandle,
    webview: tauri::Webview,
    state: State<'_, BrowserState>,
    url: Option<String>,
    search_engine: Option<String>,
) -> Result<TabSnapshot, String> {
    let mut span = state
        .infrastructure()
        .diagnostics()
        .span("tabs.create_ephemeral");
    let _operation = state.session().operation().await;
    let result = create_tab_inner_ext(
        &app,
        &state,
        url,
        search_engine,
        None,
        TabProfile {
            is_ephemeral: true,
            window_id: Some(crate::commands::access::caller_window(&webview)),
            ..Default::default()
        },
    );
    span.finish(result.is_ok(), 0);
    result
}

pub(super) fn create_tab_inner(
    app: &AppHandle,
    state: &BrowserState,
    url: Option<String>,
    search_engine: Option<String>,
    side: Option<crate::state::split_docking::DockSide>,
    window_id: &str,
) -> Result<TabSnapshot, String> {
    create_tab_inner_ext(
        app,
        state,
        url,
        search_engine,
        side,
        TabProfile {
            window_id: Some(window_id.to_string()),
            ..Default::default()
        },
    )
}

#[derive(Default)]
pub(super) struct TabProfile {
    pub is_ephemeral: bool,
    pub container_id: Option<String>,
    pub workspace_id: Option<String>,
    /// The window that opens the tab; the main window when not given.
    pub window_id: Option<String>,
}

pub(super) fn create_tab_inner_ext(
    app: &AppHandle,
    state: &BrowserState,
    url: Option<String>,
    search_engine: Option<String>,
    side: Option<crate::state::split_docking::DockSide>,
    profile: TabProfile,
) -> Result<TabSnapshot, String> {
    let TabProfile {
        is_ephemeral,
        container_id,
        workspace_id,
        window_id,
    } = profile;
    let window_id =
        window_id.unwrap_or_else(|| crate::state::tab_model::MAIN_WINDOW_ID.to_string());
    let window = state
        .session()
        .window(&window_id)
        .ok_or("windows.not_found")?;
    let raw_target = url.unwrap_or_else(|| DEFAULT_HOME_URL.to_string());
    let resolved_url = crate::window::url_input::resolve_browser_input(
        app,
        &raw_target,
        search_engine.as_deref().unwrap_or("google"),
    )?
    .to_string();
    let shield_enabled = *state
        .policies()
        .shield_enabled()
        .lock()
        .map_err(|e| e.to_string())?;
    if crate::privacy::adblock_engine::site_is_protected(state, &resolved_url)
        && crate::privacy::adblock_engine::should_block_url(&resolved_url, shield_enabled)
    {
        return Err("Navigation blocked by the privacy shield".to_string());
    }

    let new_tab_id = {
        let mut counter = state
            .session()
            .edit_next_tab_counter()
            .lock()
            .map_err(|e| e.to_string())?;
        let id = format!("tab-{}", *counter);
        *counter += 1;
        id
    };

    let active_ws_id = workspace_id.unwrap_or_else(|| {
        window
            .edit_active_workspace_id()
            .lock()
            .map(|ws| ws.clone())
            .unwrap_or_else(|_| DEFAULT_WORKSPACE_ID.to_string())
    });

    TabCheckpoint::commit_with_activation(state, Some((&new_tab_id, true)), || {
        let mut tab_info = if is_ephemeral {
            TabInfo::new_ephemeral(new_tab_id.clone(), resolved_url.clone(), active_ws_id, true)
        } else {
            TabInfo::new(new_tab_id.clone(), resolved_url.clone(), active_ws_id, true)
        };
        if !is_ephemeral {
            tab_info.container_id = container_id;
        }
        tab_info.window_id.clone_from(&window_id);
        state
            .session()
            .edit_tabs()
            .lock()
            .map_err(|e| e.to_string())?
            .push(tab_info);
        if let Some(side) = side {
            if let Some(layout) = state
                .session()
                .edit_split_layouts()
                .lock()
                .map_err(|e| e.to_string())?
                .get_mut(
                    &*window
                        .edit_active_workspace_id()
                        .lock()
                        .map_err(|e| e.to_string())?,
                )
            {
                layout.group_mut(side).push(new_tab_id.clone());
            }
        }
        state.select_tab(&new_tab_id)?;
        Ok(())
    })?;
    reconcile_committed(app, state);

    let snapshot = crate::state::publish::publish_tabs(app, state, &window_id);
    crate::error_log::ignored(crate::security::emit_shell_to(
        app,
        &window_id,
        "content-url-changed",
        &resolved_url,
    ));

    Ok(snapshot)
}

#[tauri::command]
pub async fn switch_tab(
    app: AppHandle,
    state: State<'_, BrowserState>,
    tab_id: String,
) -> Result<TabSnapshot, String> {
    let mut span = state.infrastructure().diagnostics().span("tabs.switch");
    let mut wait = state
        .infrastructure()
        .diagnostics()
        .span("tabs.switch.wait");
    let _operation = state.session().operation().await;
    wait.finish(true, 0);
    drop(wait);
    let mut execution = state
        .infrastructure()
        .diagnostics()
        .span("tabs.switch.execute");
    let result = switch_tab_inner(&app, &state, tab_id);
    execution.finish(result.is_ok(), 0);
    span.finish(result.is_ok(), 0);
    result
}

pub(super) fn switch_tab_inner(
    app: &AppHandle,
    state: &BrowserState,
    tab_id: String,
) -> Result<TabSnapshot, String> {
    let window_id = state.window_of_tab(&tab_id)?;
    let previous_active_id = state
        .session()
        .window(&window_id)
        .ok_or("windows.not_found")?
        .edit_active_tab_id()
        .lock()
        .map_err(|e| e.to_string())?
        .clone();
    let previous_bounds = if crate::window::fullscreen::all(app).is_empty() {
        crate::window::viewport::content_bounds(app, state).ok()
    } else {
        None
    };
    if previous_active_id != tab_id {
        if let Some(label) = crate::window::fullscreen::active_in(app, &window_id) {
            crate::window::fullscreen::exit(app, &label, true);
        }
    }
    let activation = (previous_active_id != tab_id).then_some((tab_id.as_str(), false));
    let active_url = TabCheckpoint::commit_with_activation(state, activation, || {
        let selected = state.select_tab(&tab_id)?;
        Ok(selected.url)
    })?;
    reconcile_committed_with_bounds(app, state, previous_bounds.as_deref());

    let snapshot = crate::state::publish::publish_tabs(app, state, &window_id);
    let stats = state.snapshot_privacy_stats();
    crate::security::emit_shell_all(app, "privacy-stats-updated", &stats);
    if !active_url.is_empty() {
        crate::error_log::ignored(crate::security::emit_shell_to(
            app,
            &window_id,
            "content-url-changed",
            &active_url,
        ));
    }

    Ok(snapshot)
}

#[tauri::command]
pub async fn close_tab(
    app: AppHandle,
    state: State<'_, BrowserState>,
    tab_id: String,
    search_engine: Option<String>,
    focus_content: Option<bool>,
) -> Result<TabSnapshot, String> {
    let _operation = state.session().operation().await;
    let window_id = state.window_of_tab(&tab_id).unwrap_or_default();
    let restore_focus = focus_content.unwrap_or(false)
        && state
            .window_of_tab(&tab_id)
            .ok()
            .and_then(|window| state.session().window(&window))
            .is_some_and(|window| {
                window
                    .edit_active_tab_id()
                    .lock()
                    .is_ok_and(|id| *id == tab_id)
            });
    let snapshot = super::closed_tabs::close_one(&app, &state, &tab_id, search_engine.as_deref())?;
    if restore_focus {
        crate::window::webview_manager::focus_active_content_webview(&app, &window_id)?;
    }
    Ok(snapshot)
}

#[tauri::command]
pub async fn hibernate_tab(
    app: AppHandle,
    webview: tauri::Webview,
    state: State<'_, BrowserState>,
    tab_id: String,
) -> Result<TabSnapshot, String> {
    let window_id = crate::commands::access::caller_window(&webview);
    let _operation = state.session().operation().await;
    let before = TabCheckpoint::capture(&state)?;
    let changed = (|| -> Result<bool, String> {
        if state.active_tab_ids().contains(&tab_id) {
            return Err("Cannot hibernate the currently active tab".to_string());
        }

        if let Ok(mut tabs) = state.session().edit_tabs().lock() {
            if let Some(tab) = tabs.iter_mut().find(|t| t.id == tab_id) {
                if tab.protected_from_hibernation() {
                    return Err("This tab is protected from hibernation".into());
                }
                tab.is_hibernated = true;
            }
        }

        Ok(true)
    })();
    finish_change(&app, &state, before, changed)?;

    let snapshot = crate::state::publish::publish_tabs(&app, &state, &window_id);
    crate::security::emit_shell_all(
        &app,
        "privacy-stats-updated",
        state.snapshot_privacy_stats(),
    );

    Ok(snapshot)
}

#[tauri::command]
pub async fn hibernate_inactive_tabs(
    app: AppHandle,
    webview: tauri::Webview,
    state: State<'_, BrowserState>,
    min_idle_secs: Option<u64>,
) -> Result<TabSnapshot, String> {
    let window_id = crate::commands::access::caller_window(&webview);
    let _operation = state.session().operation().await;
    let before = TabCheckpoint::capture(&state)?;
    let changed = (|| -> Result<bool, String> {
        let threshold = min_idle_secs.unwrap_or(0);
        let now = TabInfo::current_epoch_secs();
        let active_ids = state.active_tab_ids();

        let mut changed = false;
        if let Ok(mut tabs) = state.session().edit_tabs().lock() {
            for tab in tabs.iter_mut() {
                if !active_ids.contains(&tab.id)
                    && !tab.protected_from_hibernation()
                    && !tab.is_hibernated
                    && now.saturating_sub(tab.last_active_epoch) >= threshold
                {
                    tab.is_hibernated = true;
                    changed = true;
                }
            }
        }

        changed |= crate::state::memory_policy::enforce_resident_limit(
            &mut state
                .session()
                .edit_tabs()
                .lock()
                .map_err(|e| e.to_string())?,
            state.session().effective_resident_limit(),
            &state
                .session()
                .edit_hibernation_exemptions()
                .lock()
                .map_err(|e| e.to_string())?,
        ) > 0;
        Ok(changed)
    })();
    finish_change(&app, &state, before, changed)?;
    let snapshot = crate::state::publish::publish_tabs(&app, &state, &window_id);
    crate::security::emit_shell_all(
        &app,
        "privacy-stats-updated",
        state.snapshot_privacy_stats(),
    );

    Ok(snapshot)
}

/// Puts the tabs of the workspaces the window is not showing to sleep (see
/// `memory_policy::hibernate_other_workspaces`). The shell asks for it a moment after a workspace switch.
#[tauri::command]
pub async fn hibernate_other_workspaces(
    app: AppHandle,
    webview: tauri::Webview,
    state: State<'_, BrowserState>,
) -> Result<TabSnapshot, String> {
    let window_id = crate::commands::access::caller_window(&webview);
    let _operation = state.session().operation().await;
    let before = TabCheckpoint::capture(&state)?;
    let changed = (|| -> Result<bool, String> {
        let active_workspace = state
            .session()
            .window(&window_id)
            .ok_or("windows.not_found")?
            .active_workspace_id()
            .lock()
            .map_err(|e| e.to_string())?
            .clone();
        let exemptions = state
            .session()
            .edit_hibernation_exemptions()
            .lock()
            .map_err(|e| e.to_string())?
            .clone();
        let hibernated = crate::state::memory_policy::hibernate_other_workspaces(
            &mut state
                .session()
                .edit_tabs()
                .lock()
                .map_err(|e| e.to_string())?,
            &window_id,
            &active_workspace,
            &exemptions,
        );
        Ok(hibernated > 0)
    })();
    finish_change(&app, &state, before, changed)?;
    let snapshot = crate::state::publish::publish_tabs(&app, &state, &window_id);
    crate::security::emit_shell_all(
        &app,
        "privacy-stats-updated",
        state.snapshot_privacy_stats(),
    );
    Ok(snapshot)
}

#[tauri::command]
pub async fn auto_hibernate_inactive_tabs(
    app: AppHandle,
    webview: tauri::Webview,
    state: State<'_, BrowserState>,
    mode: String,
) -> Result<TabSnapshot, String> {
    let window_id = crate::commands::access::caller_window(&webview);
    let _operation = state.session().operation().await;
    let before = TabCheckpoint::capture(&state)?;
    let changed = (|| -> Result<bool, String> {
        const FIVE_HOURS_SECS: u64 = 5 * 60 * 60;
        let mode = match mode.as_str() {
            "5" => Some((5 * 60, false)),
            "10" => Some((10 * 60, false)),
            "30" => Some((30 * 60, false)),
            "auto" => Some((10 * 60, true)),
            "off" => None,
            _ => return Err("Invalid automatic hibernation mode".to_string()),
        };

        let Some((regular_threshold, adaptive)) = mode else {
            let changed = crate::state::memory_policy::enforce_resident_limit(
                &mut state
                    .session()
                    .edit_tabs()
                    .lock()
                    .map_err(|e| e.to_string())?,
                state.session().effective_resident_limit(),
                &state
                    .session()
                    .edit_hibernation_exemptions()
                    .lock()
                    .map_err(|e| e.to_string())?,
            );
            return Ok(changed > 0);
        };

        let now = TabInfo::current_epoch_secs();
        let exemptions = state
            .session()
            .edit_hibernation_exemptions()
            .lock()
            .map_err(|e| e.to_string())?
            .clone();
        let active_ids = state.active_tab_ids();
        let mut frequent_tabs = std::collections::HashSet::new();

        if adaptive {
            if let Ok(db_guard) = state.infrastructure().db().lock() {
                if let Some(conn) = db_guard.as_ref() {
                    tab_usage_repo::prune_expired(conn, now)?;
                    frequent_tabs = tab_usage_repo::frequent_tab_ids(conn, now)?;
                }
            }
        }

        let mut changed = false;
        if let Ok(mut tabs) = state.session().edit_tabs().lock() {
            for tab in tabs.iter_mut() {
                if active_ids.contains(&tab.id)
                    || tab.is_hibernated
                    || tab.protected_from_hibernation()
                    || crate::state::tab_model::hibernation_domain_matches(&tab.url, &exemptions)
                {
                    continue;
                }
                let threshold = if adaptive && frequent_tabs.contains(&tab.id) {
                    FIVE_HOURS_SECS
                } else {
                    regular_threshold
                };
                if now.saturating_sub(tab.last_active_epoch) >= threshold {
                    tab.is_hibernated = true;
                    changed = true;
                }
            }
        }

        changed |= crate::state::memory_policy::enforce_resident_limit(
            &mut state
                .session()
                .edit_tabs()
                .lock()
                .map_err(|e| e.to_string())?,
            state.session().effective_resident_limit(),
            &exemptions,
        ) > 0;
        Ok(changed)
    })();
    finish_change(&app, &state, before, changed)?;
    let snapshot = crate::state::publish::publish_tabs(&app, &state, &window_id);
    crate::security::emit_shell_all(
        &app,
        "privacy-stats-updated",
        state.snapshot_privacy_stats(),
    );
    Ok(snapshot)
}

#[tauri::command]
pub async fn reorder_tabs(
    app: AppHandle,
    webview: tauri::Webview,
    state: State<'_, BrowserState>,
    tab_ids: Vec<String>,
) -> Result<TabSnapshot, String> {
    let window_id = crate::commands::access::caller_window(&webview);
    let _operation = state.session().operation().await;
    TabCheckpoint::commit(&state, || {
        {
            if let Ok(mut tabs) = state.session().edit_tabs().lock() {
                let previous = std::mem::take(&mut *tabs);
                let original_ids: Vec<_> = previous.iter().map(|tab| tab.id.clone()).collect();
                let mut by_id: std::collections::HashMap<_, _> = previous
                    .into_iter()
                    .map(|tab| (tab.id.clone(), tab))
                    .collect();
                let mut ordered = Vec::with_capacity(by_id.len());
                for id in tab_ids.iter().chain(original_ids.iter()) {
                    if let Some(tab) = by_id.remove(id) {
                        ordered.push(tab);
                    }
                }
                *tabs = ordered;
            }
        }

        Ok(())
    })?;
    reconcile_committed(&app, &state);
    Ok(crate::state::publish::publish_tabs(
        &app, &state, &window_id,
    ))
}

#[tauri::command]
pub async fn move_tab_to_workspace(
    app: AppHandle,
    webview: tauri::Webview,
    state: State<'_, BrowserState>,
    tab_id: String,
    workspace_id: String,
) -> Result<TabSnapshot, String> {
    let window_id = crate::commands::access::caller_window(&webview);
    let _operation = state.session().operation().await;
    TabCheckpoint::commit(&state, || {
        state.move_tab(&tab_id, &workspace_id)?;

        Ok(())
    })?;
    // Reconciliation compares the actual resident profile with committed metadata.
    reconcile_committed(&app, &state);
    Ok(crate::state::publish::publish_tabs(
        &app, &state, &window_id,
    ))
}

#[tauri::command]
pub async fn set_resident_limit(
    app: AppHandle,
    webview: tauri::Webview,
    state: State<'_, BrowserState>,
    limit: usize,
    enforce: Option<bool>,
) -> Result<TabSnapshot, String> {
    let window_id = crate::commands::access::caller_window(&webview);
    if !(1..=100).contains(&limit) {
        return Err("Resident limit must be between 1 and 100".into());
    }
    let _operation = state.session().operation().await;
    TabCheckpoint::commit(&state, || {
        state.session().set_resident_limit(limit);
        state
            .session()
            .set_resident_limit_enforced(enforce.unwrap_or(true));
        Ok(())
    })?;
    reconcile_committed(&app, &state);
    Ok(crate::state::publish::publish_tabs(
        &app, &state, &window_id,
    ))
}

#[tauri::command]
pub async fn set_hibernation_exemptions(
    app: AppHandle,
    webview: tauri::Webview,
    state: State<'_, BrowserState>,
    domains: Vec<String>,
) -> Result<TabSnapshot, String> {
    let window_id = crate::commands::access::caller_window(&webview);
    if domains.len() > 256 {
        return Err("settings.invalid_hibernation_domain".into());
    }
    let mut normalized = Vec::with_capacity(domains.len());
    for domain in domains {
        let domain = crate::state::tab_model::normalize_hibernation_domain(&domain)?;
        if !normalized.contains(&domain) {
            normalized.push(domain);
        }
    }
    normalized.sort();

    let _operation = state.session().operation().await;
    let previous = state
        .session()
        .edit_hibernation_exemptions()
        .lock()
        .map_err(|e| e.to_string())?
        .clone();
    *state
        .session()
        .edit_hibernation_exemptions()
        .lock()
        .map_err(|e| e.to_string())? = normalized;
    if let Err(error) = TabCheckpoint::commit(&state, || Ok(())) {
        *state
            .session()
            .edit_hibernation_exemptions()
            .lock()
            .map_err(|e| e.to_string())? = previous;
        return Err(error);
    }
    reconcile_committed(&app, &state);
    Ok(crate::state::publish::publish_tabs(
        &app, &state, &window_id,
    ))
}

#[tauri::command]
pub fn set_ask_download_destination(state: State<'_, BrowserState>, enabled: bool) {
    state.policies().set_ask_download_destination(enabled);
}

#[tauri::command]
pub async fn set_tab_pinned(
    app: AppHandle,
    webview: tauri::Webview,
    state: State<'_, BrowserState>,
    tab_id: String,
    pinned: bool,
) -> Result<TabSnapshot, String> {
    let window_id = crate::commands::access::caller_window(&webview);
    let _operation = state.session().operation().await;
    TabCheckpoint::commit(&state, || {
        state
            .session()
            .edit_tabs()
            .lock()
            .map_err(|e| e.to_string())?
            .iter_mut()
            .find(|tab| tab.id == tab_id)
            .ok_or("Tab was not found")?
            .is_pinned = pinned;
        Ok(())
    })?;
    reconcile_committed(&app, &state);
    Ok(crate::state::publish::publish_tabs(
        &app, &state, &window_id,
    ))
}

/// A committed session remains authoritative if native reconciliation fails.
pub(super) fn reconcile_committed(app: &AppHandle, state: &BrowserState) {
    reconcile_committed_with_bounds(app, state, None);
}

fn reconcile_committed_with_bounds(
    app: &AppHandle,
    state: &BrowserState,
    previous: Option<&[(String, crate::state::browser_state::ViewportBounds)]>,
) {
    for (owner, window_id) in crate::window::fullscreen::all(app) {
        let selected = state
            .session()
            .window(&window_id)
            .and_then(|window| window.active_tab_id().lock().ok().map(|id| id.clone()))
            .map(|id| crate::state::tab_model::webview_label_for_tab(&id));
        if selected.as_deref() != Some(owner.as_str()) {
            crate::window::fullscreen::exit(app, &owner, true);
        }
    }
    if let Err(error) = crate::window::sync_webviews_after_selection(app, state, previous) {
        crate::error_log::ignored(crate::security::emit_shell(
            app,
            "browser-error",
            format!("The session was saved, but a page could not be displayed: {error}"),
        ));
    }
}

fn finish_change(
    app: &AppHandle,
    state: &BrowserState,
    before: TabCheckpoint,
    changed: Result<bool, String>,
) -> Result<(), String> {
    match changed.and_then(|changed| {
        if changed {
            state.persist_session()?;
        }
        Ok(changed)
    }) {
        Ok(true) => reconcile_committed(app, state),
        Ok(false) => {}
        Err(error) => {
            before.restore(state)?;
            return Err(error);
        }
    }
    Ok(())
}

#[cfg(test)]
mod popup_tests {
    use super::*;

    #[test]
    fn popup_policy_allows_only_credential_free_web_urls_when_enabled() {
        for value in ["https://example.test/path", "http://example.test/"] {
            assert!(popup_target_allowed(true, &value.parse().unwrap()));
        }
        for value in [
            "about:blank",
            "javascript:alert(1)",
            "file:///tmp/file",
            "https://user:pass@example.test/",
        ] {
            assert!(!popup_target_allowed(true, &value.parse().unwrap()));
        }
        assert!(!popup_target_allowed(
            false,
            &"https://example.test/".parse().unwrap()
        ));
    }
}
