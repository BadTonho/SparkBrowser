use rusqlite::Connection;
use std::sync::{atomic::AtomicBool, Mutex};

/// Domain-owned state. Storage cannot be accessed through BrowserState fields.
pub(crate) struct Infrastructure {
    startup_ready: AtomicBool,
    diagnostics: crate::state::diagnostics::Diagnostics,
    db: Mutex<Option<Connection>>,
}

impl Infrastructure {
    pub(crate) fn startup_ready(&self) -> &AtomicBool {
        &self.startup_ready
    }
    pub(crate) fn diagnostics(&self) -> &crate::state::diagnostics::Diagnostics {
        &self.diagnostics
    }
    pub(crate) fn db(&self) -> &Mutex<Option<Connection>> {
        &self.db
    }
}

impl Default for Infrastructure {
    fn default() -> Self {
        Self {
            startup_ready: AtomicBool::new(false),
            diagnostics: crate::state::diagnostics::Diagnostics::default(),
            db: Mutex::new(None),
        }
    }
}
