use super::super::{
    browser_state::{DEFAULT_HOME_URL, INITIAL_TAB_ID},
    tab_model::TabInfo,
    workspace_model::{WorkspaceItem, DEFAULT_WORKSPACE_ID},
};
use std::sync::Mutex;

/// Domain-owned state. Storage cannot be accessed through BrowserState fields.
pub(crate) struct Session {
    split_layouts:
        Mutex<std::collections::HashMap<String, crate::state::split_layout::SplitLayout>>,
    split_preview: Mutex<Option<(String, f64)>>,
    resident_limit: std::sync::atomic::AtomicUsize,
    hibernation_exemptions: std::sync::Mutex<Vec<String>>,
    snapshot_revision: Mutex<u64>,
    tab_operations: tauri::async_runtime::Mutex<()>,
    workspaces: Mutex<Vec<WorkspaceItem>>,
    active_workspace_id: Mutex<String>,
    tabs: Mutex<Vec<TabInfo>>,
    active_tab_id: Mutex<String>,
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
        super::read_port::ReadPort::new(&self.active_workspace_id)
    }
    pub(in crate::state) fn edit_active_workspace_id(&self) -> &Mutex<String> {
        &self.active_workspace_id
    }
    pub(crate) fn tabs(&self) -> super::read_port::ReadPort<'_, Vec<TabInfo>> {
        super::read_port::ReadPort::new(&self.tabs)
    }
    pub(in crate::state) fn edit_tabs(&self) -> &Mutex<Vec<TabInfo>> {
        &self.tabs
    }
    pub(crate) fn active_tab_id(&self) -> super::read_port::ReadPort<'_, String> {
        super::read_port::ReadPort::new(&self.active_tab_id)
    }
    pub(in crate::state) fn edit_active_tab_id(&self) -> &Mutex<String> {
        &self.active_tab_id
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
        let initial_tab = TabInfo::new_default(
            INITIAL_TAB_ID.to_string(),
            DEFAULT_HOME_URL.to_string(),
            true,
        );
        Self {
            split_layouts: Mutex::new(Default::default()),
            split_preview: Mutex::new(None),
            resident_limit: std::sync::atomic::AtomicUsize::new(8),
            hibernation_exemptions: std::sync::Mutex::new(Vec::new()),
            snapshot_revision: Mutex::new(0),
            tab_operations: tauri::async_runtime::Mutex::new(()),
            workspaces: Mutex::new(WorkspaceItem::default_workspaces()),
            active_workspace_id: Mutex::new(DEFAULT_WORKSPACE_ID.to_string()),
            tabs: Mutex::new(vec![initial_tab]),
            active_tab_id: Mutex::new(INITIAL_TAB_ID.to_string()),
            next_tab_counter: Mutex::new(2),
        }
    }
}
