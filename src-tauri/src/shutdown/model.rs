use serde::Serialize;

#[derive(Clone, Copy, Debug, Default, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ShutdownPurpose {
    #[default]
    Exit,
    InstallUpdate,
    RestartRuntime,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct ShutdownRisk {
    pub id: String,
    pub title: String,
    pub url: String,
    pub form: bool,
    pub media: bool,
    pub native_download: bool,
}
#[derive(Clone, Debug, Default, Serialize, PartialEq, Eq)]
pub struct ShutdownReview {
    pub revision: u64,
    pub tabs: Vec<ShutdownRisk>,
    pub downloads: Vec<String>,
    pub native_downloads: Vec<String>,
}
impl ShutdownReview {
    pub fn has_risks(&self) -> bool {
        !self.tabs.is_empty() || !self.downloads.is_empty() || !self.native_downloads.is_empty()
    }
    pub fn same_activity(&self, other: &Self) -> bool {
        self.tabs == other.tabs
            && self.downloads == other.downloads
            && self.native_downloads == other.native_downloads
    }
}
#[derive(Clone, Debug, Default, Serialize)]
pub struct ShutdownState {
    pub purpose: ShutdownPurpose,
    pub revision: u64,
    pub phase: String,
    pub attempt: Option<String>,
    pub update_version: Option<String>,
    pub review: Option<ShutdownReview>,
    pub error: Option<String>,
}
