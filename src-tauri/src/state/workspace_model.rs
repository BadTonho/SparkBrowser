use serde::{Deserialize, Serialize};

pub const DEFAULT_WORKSPACE_ID: &str = "ws-personal";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceItem {
    pub id: String,
    pub name: String,
    pub icon: String,
    pub accent_primary: String,
    pub accent_secondary: String,
}

impl WorkspaceItem {
    pub fn default_workspaces() -> Vec<Self> {
        vec![
            Self {
                id: "ws-personal".to_string(),
                name: "Personal".to_string(),
                icon: "🌟".to_string(),
                accent_primary: "#6e8efb".to_string(),
                accent_secondary: "#a777e3".to_string(),
            },
            Self {
                id: "ws-work".to_string(),
                name: "Work".to_string(),
                icon: "💼".to_string(),
                accent_primary: "#38bdf8".to_string(),
                accent_secondary: "#6366f1".to_string(),
            },
            Self {
                id: "ws-study".to_string(),
                name: "Study".to_string(),
                icon: "📚".to_string(),
                accent_primary: "#34d399".to_string(),
                accent_secondary: "#059669".to_string(),
            },
            Self {
                id: "ws-gaming".to_string(),
                name: "Gaming".to_string(),
                icon: "🎮".to_string(),
                accent_primary: "#f43f5e".to_string(),
                accent_secondary: "#d946ef".to_string(),
            },
        ]
    }
}

