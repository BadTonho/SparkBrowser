use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UpdateChannel {
    #[default]
    Stable,
    Beta,
}
impl UpdateChannel {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Stable => "stable",
            Self::Beta => "beta",
        }
    }
    pub fn initial(version: &str) -> Self {
        if version.contains("-beta.") {
            Self::Beta
        } else {
            Self::Stable
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct UpdateInfo {
    pub version: String,
    pub tag: String,
    pub notes: String,
    pub published_at: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct UpdateState {
    pub revision: u64,
    pub supported: bool,
    pub message: String,
    pub current_version: String,
    pub channel: UpdateChannel,
    pub last_checked: Option<i64>,
    pub status: String,
    /// Download in the background and install at the next start without asking.
    pub auto_update: bool,
    /// The downloaded update is saved on disk and installs the next time Spark opens.
    pub staged: bool,
    pub update: Option<UpdateInfo>,
    pub received: u64,
    pub total: Option<u64>,
    pub error: Option<String>,
    pub retry_after: Option<i64>,
}
