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
                accent_primary: "#f5f5f5".to_string(),
                accent_secondary: "#a3a3a3".to_string(),
            },
            Self {
                id: "ws-work".to_string(),
                name: "Work".to_string(),
                icon: "💼".to_string(),
                accent_primary: "#e5e5e5".to_string(),
                accent_secondary: "#8a8a8a".to_string(),
            },
            Self {
                id: "ws-study".to_string(),
                name: "Study".to_string(),
                icon: "📚".to_string(),
                accent_primary: "#d4d4d4".to_string(),
                accent_secondary: "#999999".to_string(),
            },
            Self {
                id: "ws-gaming".to_string(),
                name: "Gaming".to_string(),
                icon: "🎮".to_string(),
                accent_primary: "#c7c7c7".to_string(),
                accent_secondary: "#737373".to_string(),
            },
        ]
    }
}

