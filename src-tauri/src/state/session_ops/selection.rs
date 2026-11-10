use crate::state::{browser_state::BrowserState, tab_checkpoint::TabCheckpoint};

impl BrowserState {
    /// Caller holds the session operation guard. A failed focus/navigation
    /// activation restores the confirmed selection without rebuilding documents.
    pub(crate) fn select_tab_committed(&self, id: &str) -> Result<(), String> {
        TabCheckpoint::commit_with_activation(self, Some((id, false)), || {
            self.select_tab(id).map(|_| ())
        })
    }
}
