use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum DownloadStatus {
    Created,
    Queued,
    Connecting,
    Retrying,
    Downloading,
    Paused,
    Complete,
    Failed,
    Cancelled,
}

impl DownloadStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            DownloadStatus::Created => "Created",
            DownloadStatus::Queued => "Queued",
            DownloadStatus::Connecting => "Connecting",
            DownloadStatus::Retrying => "Retrying",
            DownloadStatus::Downloading => "Downloading",
            DownloadStatus::Paused => "Paused",
            DownloadStatus::Complete => "Complete",
            DownloadStatus::Failed => "Failed",
            DownloadStatus::Cancelled => "Cancelled",
        }
    }
}
