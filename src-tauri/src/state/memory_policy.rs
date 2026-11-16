use super::tab_model::TabInfo;

/// A soft cap: protected tabs are never discarded to satisfy the target.
pub fn enforce_resident_limit(tabs: &mut [TabInfo], limit: usize, exemptions: &[String]) -> usize {
    let mut resident = tabs.iter().filter(|tab| !tab.is_hibernated).count();
    let mut candidates: Vec<_> = tabs
        .iter_mut()
        .filter(|tab| {
            !tab.is_hibernated
                && !tab.protected_from_hibernation()
                && !super::tab_model::hibernation_domain_matches(&tab.url, exemptions)
        })
        .collect();
    candidates.sort_by(|a, b| {
        a.last_active_epoch
            .cmp(&b.last_active_epoch)
            .then(a.id.cmp(&b.id))
    });
    let mut hibernated = 0;
    for tab in candidates {
        if resident <= limit.max(1) {
            break;
        }
        tab.is_hibernated = true;
        resident -= 1;
        hibernated += 1;
    }
    hibernated
}

/// Puts to sleep every tab of `window_id` that belongs to a workspace other than the one the window
/// shows. Tabs that are protected (media, forms, downloads, pins, ...) and exempt domains stay awake.
pub fn hibernate_other_workspaces(
    tabs: &mut [TabInfo],
    window_id: &str,
    active_workspace: &str,
    exemptions: &[String],
) -> usize {
    let mut hibernated = 0;
    for tab in tabs.iter_mut().filter(|tab| {
        tab.window_id == window_id
            && tab.workspace_id != active_workspace
            && !tab.is_hibernated
            && !tab.protected_from_hibernation()
            && !super::tab_model::hibernation_domain_matches(&tab.url, exemptions)
    }) {
        tab.is_hibernated = true;
        hibernated += 1;
    }
    hibernated
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn evicts_lru_and_preserves_forms_media_pins_downloads_and_active_tab() {
        let mut tabs: Vec<_> = (0..7)
            .map(|id| {
                let mut t = TabInfo::new_default(
                    format!("tab-{id}"),
                    "https://example.test".into(),
                    id == 0,
                );
                t.last_active_epoch = id;
                t
            })
            .collect();
        tabs[1].activity.dirty_form = true;
        tabs[2].activity.playing_media = true;
        tabs[3].is_pinned = true;
        tabs[4].activity.downloading = true;
        assert_eq!(enforce_resident_limit(&mut tabs, 6, &[]), 1);
        assert!(tabs[5].is_hibernated);
        assert!(!tabs[6].is_hibernated);
        assert_eq!(enforce_resident_limit(&mut tabs, 1, &[]), 1);
        assert!(tabs[..5].iter().all(|tab| !tab.is_hibernated));
    }
    #[test]
    fn leaving_a_workspace_hibernates_only_its_unprotected_tabs() {
        let make = |id: &str, workspace: &str, url: &str| {
            let mut tab = TabInfo::new_default(id.into(), url.into(), false);
            tab.workspace_id = workspace.into();
            tab.window_id = "main".into();
            tab
        };
        let mut tabs = vec![
            make("here", "ws-a", "https://example.test"),
            make("plain", "ws-b", "https://example.test"),
            make("media", "ws-b", "https://example.test"),
            make("form", "ws-b", "https://example.test"),
            make("pinned", "ws-b", "https://example.test"),
            make("download", "ws-b", "https://example.test"),
            make("exempt", "ws-b", "https://mail.keep.test/inbox"),
            make("other-window", "ws-b", "https://example.test"),
            make("already", "ws-c", "https://example.test"),
        ];
        tabs[2].activity.playing_media = true;
        tabs[3].activity.dirty_form = true;
        tabs[4].is_pinned = true;
        tabs[5].activity.downloading = true;
        tabs[7].window_id = "second".into();
        tabs[8].is_hibernated = true;
        let hibernated =
            hibernate_other_workspaces(&mut tabs, "main", "ws-a", &["keep.test".to_string()]);
        assert_eq!(hibernated, 1);
        let asleep: Vec<_> = tabs
            .iter()
            .filter(|tab| tab.is_hibernated)
            .map(|tab| tab.id.as_str())
            .collect();
        assert_eq!(asleep, ["plain", "already"]);
        assert_eq!(
            hibernate_other_workspaces(&mut tabs, "main", "ws-a", &[]),
            1,
            "the exempt tab is eligible without the exemption"
        );
        assert!(tabs[6].is_hibernated && !tabs[0].is_hibernated);
    }
}
