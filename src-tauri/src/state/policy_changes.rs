use super::{browser_state::BrowserState, tab_checkpoint::TabCheckpoint};

impl BrowserState {
    /// A failed preference write must not leave session fallback metadata committed.
    pub(crate) fn commit_content_policy(
        &self,
        key: &str,
        encoded: &str,
        shield_enabled: bool,
    ) -> Result<(), String> {
        TabCheckpoint::commit_transaction(
            self,
            None,
            || self.prepare_content_reset(shield_enabled),
            |conn| crate::database::preferences_repo::store(conn, key, encoded),
        )
    }
}
