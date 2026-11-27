use super::model::*;
use std::collections::HashSet;

pub fn page_url(raw: &str) -> Result<String, String> {
    let url = url::Url::parse(raw).map_err(|_| "backup.invalid_url")?;
    if raw.len() > 8192
        || !matches!(url.scheme(), "http" | "https")
        || url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
    {
        return Err("backup.invalid_url".into());
    }
    Ok(url.to_string())
}
pub fn tab_url(raw: &str) -> Result<String, String> {
    if crate::window::new_tab::is_new_tab(raw) {
        // The old address is rewritten, so imported tabs never keep it.
        Ok(crate::window::new_tab::URL.into())
    } else if crate::window::settings_tab::is_settings_url(raw) {
        Ok(crate::window::settings_tab::URL.into())
    } else if crate::window::wallpaper_tab::is_wallpaper_url(raw) {
        Ok(crate::window::wallpaper_tab::URL.into())
    } else {
        page_url(raw)
    }
}
pub const SHORTCUT_IDS: &[&str] = &[
    "zoom-in",
    "zoom-out",
    "zoom-reset",
    "private-window",
    "new-window",
    "reopen-tab",
    "switch-tabs",
    "next-tab",
    "previous-tab",
    "workspace-picker",
    "switch-workspaces",
    "move-tab-up",
    "move-tab-down",
    "move-tab-up-ctrl-shift",
    "move-tab-down-ctrl-shift",
    "new-tab",
    "navigate-back",
    "navigate-forward",
    "command-bar",
    "edit-url",
    "focus-address",
    "sidebar",
    "split-view",
    "zen",
    "history",
    "downloads",
    "settings",
    "close-tab",
    "reload-tab",
    "toggle-pip",
];

pub fn preferences(p: &BackupPreferences) -> Result<(), String> {
    if p.site_zoom.len() > 100_000 {
        return Err("backup.too_many_items".into());
    }
    let mut origins = HashSet::new();
    for row in &p.site_zoom {
        crate::database::site_zoom_repo::validate(row).map_err(|_| "backup.invalid_preferences")?;
        if !origins.insert(&row.origin) {
            return Err("backup.invalid_preferences".into());
        }
    }
    let mut bindings = HashSet::new();
    let mut hibernation_domains = HashSet::new();
    if p.hibernation_exemptions.len() > 256 {
        return Err("backup.invalid_preferences".into());
    }
    for domain in &p.hibernation_exemptions {
        let normalized = crate::state::tab_model::normalize_hibernation_domain(domain)
            .map_err(|_| "backup.invalid_preferences")?;
        if normalized != *domain || !hibernation_domains.insert(domain) {
            return Err("backup.invalid_preferences".into());
        }
    }
    if !["popup", "tab"].contains(&p.settings_display.as_str())
        || !["horizontal", "vertical"].contains(&p.tab_layout.as_str())
        || p.overlay_blur_px > 24
        || !["toolbar", "popup"].contains(&p.workspace_switcher_display.as_str())
        || !crate::search_engines::is_valid(&p.default_search_engine)
        || !["system", "pt-BR", "en"].contains(&p.language.as_str())
        || !(1..=100).contains(&p.resident_limit)
        || ![
            serde_json::json!(5),
            serde_json::json!(10),
            serde_json::json!(30),
            serde_json::json!("auto"),
            serde_json::json!("off"),
        ]
        .contains(&p.auto_hibernate_mode)
    {
        return Err("backup.invalid_preferences".into());
    }
    for (id, values) in &p.keyboard_shortcuts {
        if !SHORTCUT_IDS.contains(&id.as_str()) || values.len() != 1 {
            return Err("backup.invalid_shortcuts".into());
        }
        let b = &values[0];
        if b.key.is_empty()
            || b.key.len() > 40
            || (id == "workspace-picker"
                && (!["Alt", "Control", "Shift", "Meta"].contains(&b.key.as_str())
                    || b.ctrl
                    || b.alt
                    || b.shift
                    || b.meta))
            || (["switch-tabs", "switch-workspaces"].contains(&id.as_str()) && b.key != "Digit")
            || !bindings.insert((b.key.to_lowercase(), b.ctrl, b.alt, b.shift, b.meta))
        {
            return Err("backup.invalid_shortcuts".into());
        }
    }
    Ok(())
}
fn name(value: &str) -> Result<(), String> {
    if value.trim().is_empty() || value.chars().count() > 100 {
        Err("backup.invalid_name".into())
    } else {
        Ok(())
    }
}
pub fn document(doc: &BackupDocument) -> Result<(), String> {
    if !is_backup_format(&doc.format) || !(1..=CURRENT_VERSION).contains(&doc.version) {
        return Err("backup.unsupported_version".into());
    }
    if let Some(device) = &doc.device {
        if doc.version < 7 || !super::device::valid(device) {
            return Err("backup.invalid_document".into());
        }
    }
    preferences(&doc.preferences)?;
    let d = &doc.data;
    if d.workspaces.len()
        + d.bookmarks.bookmarks.len()
        + d.bookmarks.folders.len()
        + d.sessions.len()
        + d.notes.len()
        + d.site_search.len()
        + d.tabs.len()
        > 100_000
    {
        return Err("backup.too_many_items".into());
    }
    let mut ws = HashSet::new();
    for row in &d.workspaces {
        name(&row.name)?;
        if row.id.len() > 200
            || !ws.insert(row.id.as_str())
            || row.icon.chars().count() > 16
            || [&row.accent_primary, &row.accent_secondary]
                .iter()
                .any(|color| {
                    color.len() != 7
                        || !color.starts_with('#')
                        || !color[1..].bytes().all(|b| b.is_ascii_hexdigit())
                })
        {
            return Err("backup.invalid_workspace".into());
        }
    }
    let mut folders = HashSet::new();
    for row in &d.bookmarks.folders {
        name(&row.name)?;
        if !folders.insert(row.id) {
            return Err("backup.duplicate_id".into());
        }
    }
    let mut bookmarks = HashSet::new();
    for row in &d.bookmarks.bookmarks {
        page_url(&row.url)?;
        if !bookmarks.insert(row.id)
            || row.title.len() > 8192
            || row.icon.len() > 128
            || row.folder_id.is_some_and(|id| !folders.contains(&id))
        {
            return Err("backup.invalid_bookmark".into());
        }
    }
    for note in &d.notes {
        page_url(&note.url)?;
        if note.text.len() > 200_000 || note.title.len() > 8192 {
            return Err("backup.invalid_note".into());
        }
    }
    for search in &d.site_search {
        crate::database::site_search_repo::validate(search)?;
    }
    for session in &d.sessions {
        name(&session.name)?;
        if session.pages.is_empty()
            || session.pages.len() > 10_000
            || session.pages.iter().filter(|p| p.is_selected).count() > 1
        {
            return Err("backup.invalid_session".into());
        }
        for page in &session.pages {
            tab_url(&page.url)?;
            if page.title.len() > 8192 {
                return Err("backup.invalid_session".into());
            }
        }
    }
    let mut ids = HashSet::new();
    for tab in &d.tabs {
        tab_url(&tab.url)?;
        if tab.id.len() > 200
            || tab.title.len() > 8192
            || !ws.contains(tab.workspace_id.as_str())
            || !ids.insert(tab.id.as_str())
        {
            return Err("backup.invalid_tab".into());
        }
    }
    if d.tabs.iter().filter(|t| t.is_active).count() > 1 {
        return Err("backup.invalid_selection".into());
    }
    let tabs: Vec<_> = d.tabs.iter().map(BackupTab::tab).collect();
    let mut splits = HashSet::new();
    for layout in &d.split_layouts {
        if !(0.2..=0.8).contains(&layout.ratio)
            || !layout.valid(&tabs)
            || (doc.version >= 4 && !layout.valid_groups(&tabs))
            || !splits.insert(&layout.workspace_id)
        {
            return Err("backup.invalid_split".into());
        }
    }
    Ok(())
}
