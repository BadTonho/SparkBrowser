use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ExtensionAction {
    pub default_title: Option<String>,
    pub default_popup: Option<String>,
    pub default_icon: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionManifest {
    pub manifest_version: u32,
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub permissions: Vec<String>,
    pub host_permissions: Vec<String>,
    pub action: Option<ExtensionAction>,
    pub options_page: Option<String>,
    pub icons: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExtensionDetails {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub path: String,
    pub enabled: bool,
    pub installed_at: i64,
    pub icon_data_url: Option<String>,
    pub popup_url: Option<String>,
    pub options_url: Option<String>,
    pub permissions: Vec<String>,
}

