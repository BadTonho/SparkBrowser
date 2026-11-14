use super::super::{
    browser_state::{DEFAULT_HOME_URL, INITIAL_TAB_ID},
    tab_model::TabInfo,
    workspace_model::{WorkspaceItem, DEFAULT_WORKSPACE_ID},
};
use crate::state::tab_model::MAIN_WINDOW_ID;
use std::sync::{Arc, Mutex};

/// What one browser window has selected. Tabs and workspaces are shared; each window chooses its own.
pub(crate) struct WindowState {
    active_tab_id: Mutex<String>,
    active_workspace_id: Mutex<String>,
}
impl WindowState {
    pub(crate) fn new(active_tab_id: String, active_workspace_id: String) -> Self {
        Self {
            active_tab_id: Mutex::new(active_tab_id),
            active_workspace_id: Mutex::new(active_workspace_id),
        }
    }
    pub(crate) fn active_tab_id(&self) -> super::read_port::ReadPort<'_, String> {
        super::read_port::ReadPort::new(&self.active_tab_id)
    }
    pub(in crate::state) fn edit_active_tab_id(&self) -> &Mutex<String> {
        &self.active_tab_id
    }
    pub(crate) fn active_workspace_id(&self) -> super::read_port::ReadPort<'_, String> {
        super::read_port::ReadPort::new(&self.active_workspace_id)
    }
    pub(in crate::state) fn edit_active_workspace_id(&self) -> &Mutex<String> {
        &self.active_workspace_id
    }
}

/// Domain-owned state. Storage cannot be accessed through BrowserState fields.
pub(crate) struct Session {
    split_layouts:
        Mutex<std::collections::HashMap<String, crate::state::split_layout::SplitLayout>>,
    split_preview: Mutex<Option<(String, f64)>>,
    resident_limit: std::sync::atomic::AtomicUsize,
    resident_limit_enforced: std::sync::atomic::AtomicBool,
    hibernation_exemptions: std::sync::Mutex<Vec<String>>,
    snapshot_revision: Mutex<u64>,
    tab_operations: tauri::async_runtime::Mutex<()>,
    workspaces: Mutex<Vec<WorkspaceItem>>,
    tabs: Mutex<Vec<TabInfo>>,
    /// The first window. Code that has not been made window-aware yet works on this one.
    main: Arc<WindowState>,
    /// Every open window in the order it was created, including `main`.
    windows: Mutex<Vec<(String, Arc<WindowState>)>>,
    next_tab_counter: Mutex<u64>,
}

impl Session {
    pub(in crate::state) fn edit_split_layouts(
        &self,
    ) -> &Mutex<std::collections::HashMap<String, crate::state::split_layout::SplitLayout>> {
        &self.split_layouts
    }
    pub(in crate::state) fn edit_split_preview(&self) -> &Mutex<Option<(String, f64)>> {
        &self.split_preview
    }
    pub(crate) fn resident_limit(&self) -> usize {
        self.resident_limit
            .load(std::sync::atomic::Ordering::Acquire)
    }
    /// The limit the memory policy actually applies: with automatic hibernation switched off nothing is
    /// put to sleep on its own, so the configured number is kept but not enforced.
    pub(crate) fn effective_resident_limit(&self) -> usize {
        if self.resident_limit_enforced() {
            self.resident_limit()
        } else {
            usize::MAX
        }
    }
    pub(crate) fn resident_limit_enforced(&self) -> bool {
        self.resident_limit_enforced
            .load(std::sync::atomic::Ordering::Acquire)
    }
    pub(in crate::state) fn set_resident_limit_enforced(&self, value: bool) {
        self.resident_limit_enforced
            .store(value, std::sync::atomic::Ordering::Release);
    }
    pub(in crate::state) fn set_resident_limit(&self, value: usize) {
        self.resident_limit
            .store(value, std::sync::atomic::Ordering::Release);
    }
    pub(crate) fn hibernation_exemptions(&self) -> super::read_port::ReadPort<'_, Vec<String>> {
        super::read_port::ReadPort::new(&self.hibernation_exemptions)
    }
    pub(in crate::state) fn edit_hibernation_exemptions(&self) -> &std::sync::Mutex<Vec<String>> {
        &self.hibernation_exemptions
    }
    pub(in crate::state) fn edit_snapshot_revision(&self) -> &Mutex<u64> {
        &self.snapshot_revision
    }
    pub(crate) async fn operation(&self) -> tokio::sync::MutexGuard<'_, ()> {
        self.tab_operations.lock().await
    }
    pub(crate) fn blocking_operation(&self) -> tokio::sync::MutexGuard<'_, ()> {
        self.tab_operations.blocking_lock()
    }
    pub(crate) fn workspaces(&self) -> super::read_port::ReadPort<'_, Vec<WorkspaceItem>> {
        super::read_port::ReadPort::new(&self.workspaces)
    }
    pub(in crate::state) fn edit_workspaces(&self) -> &Mutex<Vec<WorkspaceItem>> {
        &self.workspaces
    }
    pub(crate) fn active_workspace_id(&self) -> super::read_port::ReadPort<'_, String> {
        self.main.active_workspace_id()
    }
    pub(in crate::state) fn edit_active_workspace_id(&self) -> &Mutex<String> {
        self.main.edit_active_workspace_id()
    }
    /// The selection of one window, or `None` once it is closed.
    pub(crate) fn window(&self, id: &str) -> Option<Arc<WindowState>> {
        if id == MAIN_WINDOW_ID {
            return Some(self.main.clone());
        }
        self.windows
            .lock()
            .ok()?
            .iter()
            .find(|(window, _)| window == id)
            .map(|(_, state)| state.clone())
    }
    /// The window ids in creation order.
    pub(crate) fn window_ids(&self) -> Vec<String> {
        self.windows
            .lock()
            .map(|windows| windows.iter().map(|(id, _)| id.clone()).collect())
            .unwrap_or_else(|_| vec![MAIN_WINDOW_ID.to_string()])
    }
    pub(in crate::state) fn open_window(
        &self,
        id: &str,
        active_tab_id: String,
        active_workspace_id: String,
    ) -> Result<Arc<WindowState>, String> {
        let mut windows = self.windows.lock().map_err(|e| e.to_string())?;
        if let Some((_, state)) = windows.iter().find(|(window, _)| window == id) {
            return Ok(state.clone());
        }
        let state = Arc::new(WindowState::new(active_tab_id, active_workspace_id));
        windows.push((id.to_string(), state.clone()));
        Ok(state)
    }
    pub(in crate::state) fn close_window(&self, id: &str) -> Result<(), String> {
        if id == MAIN_WINDOW_ID {
            return Err("windows.cannot_close_main".into());
        }
        self.windows
            .lock()
            .map_err(|e| e.to_string())?
            .retain(|(window, _)| window != id);
        Ok(())
    }
    pub(crate) fn tabs(&self) -> super::read_port::ReadPort<'_, Vec<TabInfo>> {
        super::read_port::ReadPort::new(&self.tabs)
    }
    pub(in crate::state) fn edit_tabs(&self) -> &Mutex<Vec<TabInfo>> {
        &self.tabs
    }
    pub(crate) fn active_tab_id(&self) -> super::read_port::ReadPort<'_, String> {
        self.main.active_tab_id()
    }
    pub(in crate::state) fn edit_active_tab_id(&self) -> &Mutex<String> {
        self.main.edit_active_tab_id()
    }
    pub(crate) fn next_tab_counter(&self) -> super::read_port::ReadPort<'_, u64> {
        super::read_port::ReadPort::new(&self.next_tab_counter)
    }
    pub(in crate::state) fn edit_next_tab_counter(&self) -> &Mutex<u64> {
        &self.next_tab_counter
    }
}

impl Default for Session {
    fn default() -> Self {
        let main = Arc::new(WindowState::new(
            INITIAL_TAB_ID.to_string(),
            DEFAULT_WORKSPACE_ID.to_string(),
        ));
        let initial_tab = TabInfo::new_default(
            INITIAL_TAB_ID.to_string(),
            DEFAULT_HOME_URL.to_string(),
            true,
        );
        Self {
            split_layouts: Mutex::new(Default::default()),
            split_preview: Mutex::new(None),
            resident_limit: std::sync::atomic::AtomicUsize::new(8),
            resident_limit_enforced: std::sync::atomic::AtomicBool::new(true),
            hibernation_exemptions: std::sync::Mutex::new(Vec::new()),
            snapshot_revision: Mutex::new(0),
            tab_operations: tauri::async_runtime::Mutex::new(()),
            workspaces: Mutex::new(WorkspaceItem::default_workspaces()),
            tabs: Mutex::new(vec![initial_tab]),
            main: main.clone(),
            windows: Mutex::new(vec![(MAIN_WINDOW_ID.to_string(), main)]),
            next_tab_counter: Mutex::new(2),
        }
    }
}
