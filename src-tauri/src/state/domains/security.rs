use crate::sync::Mutex;

/// Domain-owned state. Storage cannot be accessed through BrowserState fields.
pub(crate) struct Security {
    keyboard_bridge_sequences: Mutex<crate::security::sequence::Sequences>,
    keyboard_bridge_tokens: Mutex<std::collections::HashMap<String, String>>,
    shield_script_ids: Mutex<std::collections::HashMap<String, String>>,
}

impl Security {
    pub(crate) fn keyboard_bridge_sequences(&self) -> &Mutex<crate::security::sequence::Sequences> {
        &self.keyboard_bridge_sequences
    }
    pub(crate) fn keyboard_bridge_tokens(
        &self,
    ) -> &Mutex<std::collections::HashMap<String, String>> {
        &self.keyboard_bridge_tokens
    }
    pub(crate) fn shield_script_ids(&self) -> &Mutex<std::collections::HashMap<String, String>> {
        &self.shield_script_ids
    }
}

impl Default for Security {
    fn default() -> Self {
        Self {
            keyboard_bridge_sequences: Mutex::new(Default::default()),
            keyboard_bridge_tokens: Mutex::new(std::collections::HashMap::new()),
            shield_script_ids: Mutex::new(std::collections::HashMap::new()),
        }
    }
}
