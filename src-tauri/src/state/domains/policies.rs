use std::sync::{atomic::AtomicBool, Mutex};

/// Domain-owned state. Storage cannot be accessed through BrowserState fields.
pub(crate) struct Policies {
    ask_download_destination: AtomicBool,
    #[cfg(target_os = "linux")]
    caret_browsing_enabled: AtomicBool,
    workspace_picker_hold_key: Mutex<String>,
    keyboard_shortcuts: Mutex<serde_json::Value>,
    shield_enabled: Mutex<bool>,
    history_retention_months: std::sync::atomic::AtomicU64,
    history_retention_operation: tokio::sync::Mutex<()>,
    allow_popups: AtomicBool,
    smart_screen_enabled: AtomicBool,
    smart_screen_supported: AtomicBool,
    auto_skip_ads_enabled: AtomicBool,
    workspace_isolation: AtomicBool,
    tracking_settings: Mutex<crate::privacy::tracking_parameters::TrackingSettings>,
    gpc_settings: Mutex<crate::privacy::tracking_parameters::TrackingSettings>,
    shield_settings: Mutex<crate::privacy::tracking_parameters::TrackingSettings>,
    blocked_count: Mutex<u64>,
}

impl Policies {
    pub(crate) fn ask_download_destination(&self) -> bool {
        self.ask_download_destination
            .load(std::sync::atomic::Ordering::Acquire)
    }
    pub(crate) fn set_ask_download_destination(&self, value: bool) {
        self.ask_download_destination
            .store(value, std::sync::atomic::Ordering::Release);
    }
    #[cfg(target_os = "linux")]
    pub(crate) fn caret_browsing_enabled(&self) -> bool {
        self.caret_browsing_enabled
            .load(std::sync::atomic::Ordering::Acquire)
    }
    #[cfg(target_os = "linux")]
    pub(crate) fn set_caret_browsing_enabled(&self, value: bool) {
        self.caret_browsing_enabled
            .store(value, std::sync::atomic::Ordering::Release);
    }
    pub(crate) fn workspace_picker_hold_key(&self) -> super::read_port::ReadPort<'_, String> {
        super::read_port::ReadPort::new(&self.workspace_picker_hold_key)
    }
    pub(crate) fn set_workspace_picker_hold_key(&self, value: String) -> Result<(), String> {
        *self
            .workspace_picker_hold_key
            .lock()
            .map_err(|e| e.to_string())? = value;
        Ok(())
    }
    pub(crate) fn keyboard_shortcuts(&self) -> super::read_port::ReadPort<'_, serde_json::Value> {
        super::read_port::ReadPort::new(&self.keyboard_shortcuts)
    }
    pub(crate) fn set_keyboard_shortcuts(&self, value: serde_json::Value) -> Result<(), String> {
        *self.keyboard_shortcuts.lock().map_err(|e| e.to_string())? = value;
        Ok(())
    }
    pub(crate) fn shield_enabled(&self) -> super::read_port::ReadPort<'_, bool> {
        super::read_port::ReadPort::new(&self.shield_enabled)
    }
    pub(crate) fn set_shield_enabled(&self, value: bool) -> Result<(), String> {
        *self.shield_enabled.lock().map_err(|e| e.to_string())? = value;
        Ok(())
    }
    pub(crate) fn history_retention_months(&self) -> u64 {
        self.history_retention_months
            .load(std::sync::atomic::Ordering::Acquire)
    }
    pub(crate) fn set_history_retention_months(&self, value: u64) {
        self.history_retention_months
            .store(value, std::sync::atomic::Ordering::Release);
    }
    pub(crate) fn history_retention_operation(&self) -> &tokio::sync::Mutex<()> {
        &self.history_retention_operation
    }
    pub(crate) fn allow_popups(&self) -> bool {
        self.allow_popups.load(std::sync::atomic::Ordering::Acquire)
    }
    pub(crate) fn set_allow_popups(&self, value: bool) {
        self.allow_popups
            .store(value, std::sync::atomic::Ordering::Release);
    }
    pub(crate) fn smart_screen_enabled(&self) -> bool {
        self.smart_screen_enabled
            .load(std::sync::atomic::Ordering::Acquire)
    }
    pub(crate) fn set_smart_screen_enabled(&self, value: bool) {
        self.smart_screen_enabled
            .store(value, std::sync::atomic::Ordering::Release);
    }
    pub(crate) fn smart_screen_supported(&self) -> bool {
        self.smart_screen_supported
            .load(std::sync::atomic::Ordering::Acquire)
    }
    pub(crate) fn set_smart_screen_supported(&self, value: bool) {
        self.smart_screen_supported
            .store(value, std::sync::atomic::Ordering::Release);
    }
    pub(crate) fn auto_skip_ads_enabled(&self) -> bool {
        self.auto_skip_ads_enabled
            .load(std::sync::atomic::Ordering::Acquire)
    }
    pub(crate) fn set_auto_skip_ads_enabled(&self, value: bool) {
        self.auto_skip_ads_enabled
            .store(value, std::sync::atomic::Ordering::Release);
    }
    pub(crate) fn workspace_isolation(&self) -> bool {
        self.workspace_isolation
            .load(std::sync::atomic::Ordering::Acquire)
    }
    pub(crate) fn set_workspace_isolation(&self, value: bool) {
        self.workspace_isolation
            .store(value, std::sync::atomic::Ordering::Release);
    }
    pub(crate) fn tracking_settings(
        &self,
    ) -> super::read_port::ReadPort<'_, crate::privacy::tracking_parameters::TrackingSettings> {
        super::read_port::ReadPort::new(&self.tracking_settings)
    }
    pub(crate) fn set_tracking_settings(
        &self,
        value: crate::privacy::tracking_parameters::TrackingSettings,
    ) -> Result<(), String> {
        *self.tracking_settings.lock().map_err(|e| e.to_string())? = value;
        Ok(())
    }
    pub(crate) fn gpc_settings(
        &self,
    ) -> super::read_port::ReadPort<'_, crate::privacy::tracking_parameters::TrackingSettings> {
        super::read_port::ReadPort::new(&self.gpc_settings)
    }
    pub(crate) fn set_gpc_settings(
        &self,
        value: crate::privacy::tracking_parameters::TrackingSettings,
    ) -> Result<(), String> {
        *self.gpc_settings.lock().map_err(|e| e.to_string())? = value;
        Ok(())
    }
    pub(crate) fn shield_settings(
        &self,
    ) -> super::read_port::ReadPort<'_, crate::privacy::tracking_parameters::TrackingSettings> {
        super::read_port::ReadPort::new(&self.shield_settings)
    }
    pub(crate) fn set_shield_settings(
        &self,
        value: crate::privacy::tracking_parameters::TrackingSettings,
    ) -> Result<(), String> {
        *self.shield_settings.lock().map_err(|e| e.to_string())? = value;
        Ok(())
    }
    pub(crate) fn blocked_count(&self) -> super::read_port::ReadPort<'_, u64> {
        super::read_port::ReadPort::new(&self.blocked_count)
    }
    pub(crate) fn record_blocked_request(&self) {
        if let Ok(mut count) = self.blocked_count.lock() {
            *count = count.saturating_add(1);
        }
    }
}

impl Default for Policies {
    fn default() -> Self {
        Self {
            ask_download_destination: AtomicBool::new(true),
            #[cfg(target_os = "linux")]
            caret_browsing_enabled: AtomicBool::new(false),
            workspace_picker_hold_key: Mutex::new("Alt".to_string()),
            keyboard_shortcuts: Mutex::new(serde_json::Value::Object(Default::default())),
            shield_enabled: Mutex::new(true),
            history_retention_months: std::sync::atomic::AtomicU64::new(3),
            history_retention_operation: tokio::sync::Mutex::new(()),
            allow_popups: AtomicBool::new(false),
            smart_screen_enabled: AtomicBool::new(true),
            smart_screen_supported: AtomicBool::new(false),
            auto_skip_ads_enabled: AtomicBool::new(false),
            workspace_isolation: AtomicBool::new(false),
            tracking_settings: Mutex::new(Default::default()),
            gpc_settings: Mutex::new(Default::default()),
            shield_settings: Mutex::new(crate::privacy::tracking_parameters::TrackingSettings {
                enabled: true,
                exceptions: Vec::new(),
            }),
            blocked_count: Mutex::new(0),
        }
    }
}
