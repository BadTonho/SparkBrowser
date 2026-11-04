use std::collections::{HashMap, HashSet};

#[derive(Default)]
struct Document {
    id: String,
    last: u64,
    retired: HashSet<String>,
}
#[derive(Default)]
pub struct Sequences(HashMap<String, Document>);
impl Sequences {
    pub fn remove(&mut self, label: &str) {
        self.0.remove(label);
        self.0.remove(&format!("{label}:activity"));
    }
    pub fn clear(&mut self) {
        self.0.clear();
    }
    pub fn admit(&mut self, label: &str, document: &str, sequence: u64) -> bool {
        if document.is_empty() || document.len() > 128 || sequence == 0 {
            return false;
        }
        let state = self.0.entry(label.into()).or_default();
        if state.id != document {
            if state.retired.contains(document) {
                return false;
            }
            // Keep retired documents rejected for this WebView generation.
            // The lifecycle removes the entry when the WebView is destroyed.
            if !state.id.is_empty() {
                state.retired.insert(std::mem::take(&mut state.id));
            }
            state.id = document.into();
            state.last = 0;
        }
        if sequence <= state.last {
            return false;
        }
        state.last = sequence;
        true
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn replay_and_retired_documents_cannot_repeat_actions() {
        let mut state = Sequences::default();
        assert!(state.admit("view", "first", 1));
        assert!(!state.admit("view", "first", 1));
        assert!(state.admit("view", "second", 1));
        assert!(!state.admit("view", "first", 2));
        state.remove("view");
        assert!(state.admit("view", "first", 1));
    }
}
