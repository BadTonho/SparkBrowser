use super::tab_model::TabInfo;
use serde::Serialize;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProtectionReason {
    Selection,
    Split,
    Pinned,
    Loading,
    Form,
    Media,
    Download,
}
impl TabInfo {
    /// Visibility protects both split panes, including while browser overlays hide their WebViews.
    pub fn protection_reasons(&self, in_current_split: bool) -> Vec<ProtectionReason> {
        use ProtectionReason::*;
        [
            (self.is_active, Selection),
            (
                self.is_visible && (in_current_split || !self.is_active),
                Split,
            ),
            (self.is_pinned, Pinned),
            (self.navigation.loading, Loading),
            (self.activity.dirty_form, Form),
            (self.activity.playing_media, Media),
            (self.activity.downloading, Download),
        ]
        .into_iter()
        .filter_map(|(present, reason)| present.then_some(reason))
        .collect()
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn diagnostics_and_hibernation_share_all_protection_flags() {
        let mut t = TabInfo::new_default("t".into(), "https://example.test".into(), true);
        t.is_visible = true;
        t.is_pinned = true;
        t.navigation.loading = true;
        t.activity.dirty_form = true;
        t.activity.playing_media = true;
        t.activity.downloading = true;
        assert_eq!(t.protection_reasons(true).len(), 7);
        assert!(t.protected_from_hibernation());
        t.is_active = false;
        t.is_visible = false;
        t.is_pinned = false;
        t.navigation.loading = false;
        t.activity = Default::default();
        assert!(t.protection_reasons(false).is_empty());
        assert!(!t.protected_from_hibernation());
        t.is_visible = true;
        assert_eq!(t.protection_reasons(false), vec![ProtectionReason::Split]);
        assert!(t.protected_from_hibernation());
    }
}
