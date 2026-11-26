use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadTask {
    pub id: String,
    pub revision: u64,
    pub file_name: String,
    pub url: String,
    pub source: String,
    pub source_page_url: Option<String>,
    pub status: String,
    pub received: u64,
    pub total: Option<u64>,
    pub error: Option<String>,
    pub cancellable: bool,
    pub can_pause: bool,
    pub can_resume: bool,
    pub can_restart: bool,
    pub can_verify: bool,
    pub save_path: Option<String>,
    pub expected_sha256: Option<String>,
    pub actual_sha256: Option<String>,
    pub verification_received: u64,
}
impl DownloadTask {
    pub fn pending(&self) -> bool {
        matches!(
            self.status.as_str(),
            "Queued" | "Downloading" | "Pausing" | "Cancelling" | "Verifying" | "Finalizing"
        )
    }
    pub fn unfinished(&self) -> bool {
        !matches!(self.status.as_str(), "Completed" | "Cancelled")
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadRecord {
    pub task: DownloadTask,
    pub tab_id: String,
    pub directory: PathBuf,
    pub confirmed: u64,
    pub prefix_sha256: String,
    pub final_url: Option<String>,
    pub etag: Option<String>,
    #[serde(default)]
    pub body_completed: bool,
    #[serde(default)]
    pub published: bool,
    /// The page a video came from, sent as `Referer`. Empty for every other download.
    #[serde(default)]
    pub referer: Option<String>,
}
impl DownloadRecord {
    pub fn partial(&self) -> PathBuf {
        self.directory
            .join(".spark-partials")
            .join(format!("{}.part", self.task.id))
    }
    pub fn refresh_capabilities(&mut self, native_available: bool) {
        let manual = self.task.source == "url";
        // A stream is joined from segments, so it can only start over.
        let stream = self.task.source == "hls";
        let idle = !self.task.pending();
        self.task.cancellable =
            self.task.unfinished() && (manual || stream || native_available || idle);
        self.task.can_pause = matches!(self.task.status.as_str(), "Queued" | "Downloading")
            && (manual || native_available);
        self.task.can_resume = matches!(self.task.status.as_str(), "Paused" | "Interrupted")
            && if stream {
                !self.body_completed
            } else if manual {
                !self.body_completed && (self.confirmed == 0 || self.etag.is_some())
            } else {
                native_available
            };
        self.task.can_restart =
            (manual || stream) && idle && self.task.unfinished() && !self.published;
        self.task.can_verify = idle
            && ((self.body_completed && !self.published && self.partial().is_file())
                || (self.published
                    && self
                        .task
                        .save_path
                        .as_ref()
                        .is_some_and(|p| std::path::Path::new(p).is_file())));
    }
}
pub fn checksum(value: Option<String>) -> Result<Option<String>, String> {
    value
        .filter(|s| !s.trim().is_empty())
        .map(|s| {
            let s = s.trim().to_ascii_lowercase();
            if s.len() != 64 || !s.bytes().all(|b| b.is_ascii_hexdigit()) {
                return Err("downloads.invalid_checksum".into());
            }
            Ok(s)
        })
        .transpose()
}
