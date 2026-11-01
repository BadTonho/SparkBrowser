use super::tab_model::TabInfo;

/// A soft cap: protected tabs are never discarded to satisfy the target.
pub fn enforce_resident_limit(tabs: &mut [TabInfo], limit: usize) -> usize {
    let mut resident = tabs.iter().filter(|tab| !tab.is_hibernated).count();
    let mut candidates: Vec<_> = tabs
        .iter_mut()
        .filter(|tab| !tab.is_hibernated && !tab.protected_from_hibernation())
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
        assert_eq!(enforce_resident_limit(&mut tabs, 6), 1);
        assert!(tabs[5].is_hibernated);
        assert!(!tabs[6].is_hibernated);
        assert_eq!(enforce_resident_limit(&mut tabs, 1), 1);
        assert!(tabs[..5].iter().all(|tab| !tab.is_hibernated));
    }
}
