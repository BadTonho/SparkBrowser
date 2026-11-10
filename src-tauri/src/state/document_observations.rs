use super::{
    browser_state::BrowserState,
    tab_model::{NavigationState, TabActivity},
};

/// Native callbacks can report document state, but cannot reorganize tabs or profiles.
#[derive(Clone, PartialEq)]
pub(crate) struct DocumentObservation {
    pub url: String,
    pub title: String,
    pub navigation: NavigationState,
    pub activity: TabActivity,
}

impl BrowserState {
    pub(crate) fn observe_document(
        &self,
        id: &str,
        update: impl FnOnce(&mut DocumentObservation),
    ) -> Result<Option<serde_json::Value>, String> {
        let mut tabs = self
            .session()
            .edit_tabs()
            .lock()
            .map_err(|e| e.to_string())?;
        let Some(tab) = tabs.iter_mut().find(|tab| tab.id == id) else {
            return Ok(None);
        };
        let before = DocumentObservation {
            url: tab.url.clone(),
            title: tab.title.clone(),
            navigation: tab.navigation.clone(),
            activity: tab.activity.clone(),
        };
        let mut observed = before.clone();
        update(&mut observed);
        if observed == before {
            return Ok(None);
        }
        tab.url = observed.url;
        tab.title = observed.title;
        tab.navigation = observed.navigation;
        tab.navigation.revision = before.navigation.revision + 1;
        tab.activity = observed.activity;
        Ok(Some(
            serde_json::json!({"id": tab.id, "url": tab.url, "title": tab.title,
            "navigation": tab.navigation, "activity": tab.activity}),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn observations_are_revisioned_and_closed_documents_cannot_reappear() {
        let state = BrowserState::default();
        assert!(state
            .observe_document("closed", |tab| tab.url = "https://example.test".into())
            .unwrap()
            .is_none());
        assert!(state.observe_document("tab-1", |_| {}).unwrap().is_none());
        let before = state.snapshot_tabs()[0].clone();
        let payload = state
            .observe_document("tab-1", |tab| {
                tab.activity.downloading = true;
                tab.navigation.revision = 999;
            })
            .unwrap()
            .unwrap();
        let after = state.snapshot_tabs()[0].clone();
        assert_eq!(payload["navigation"]["revision"], 1);
        assert_eq!(after.workspace_id, before.workspace_id);
        assert_eq!(after.container_id, before.container_id);
        assert_eq!(after.is_active, before.is_active);
        assert!(after.activity.downloading);
    }
}
