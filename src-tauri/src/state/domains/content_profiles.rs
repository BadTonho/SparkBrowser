use std::{collections::HashMap, path::PathBuf, sync::Mutex};

/// Actual profile associated with each resident document, independent of workspace metadata.
#[derive(Default)]
pub(crate) struct ContentProfiles(Mutex<HashMap<String, Option<PathBuf>>>);
impl ContentProfiles {
    pub(crate) fn matches(&self, label: &str, profile: &Option<PathBuf>) -> Result<bool, String> {
        Ok(self.0.lock().map_err(|e| e.to_string())?.get(label) == Some(profile))
    }
    pub(crate) fn remember(&self, label: &str, profile: Option<PathBuf>) -> Result<(), String> {
        self.0
            .lock()
            .map_err(|e| e.to_string())?
            .insert(label.into(), profile);
        Ok(())
    }
    pub(crate) fn forget(&self, label: &str) -> Result<(), String> {
        self.0.lock().map_err(|e| e.to_string())?.remove(label);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn profile_changes_and_unknown_documents_require_recreation() {
        let profiles = ContentProfiles::default();
        assert!(!profiles.matches("tab", &None).unwrap());
        profiles.remember("tab", None).unwrap();
        assert!(profiles.matches("tab", &None).unwrap());
        let account = Some(PathBuf::from("accounts/fixture"));
        assert!(!profiles.matches("tab", &account).unwrap());
        profiles.remember("tab", account.clone()).unwrap();
        assert!(profiles.matches("tab", &account).unwrap());
        assert!(!profiles
            .matches("tab", &Some(PathBuf::from("workspaces/other")))
            .unwrap());
        profiles.forget("tab").unwrap();
        assert!(!profiles.matches("tab", &account).unwrap());
    }
}
