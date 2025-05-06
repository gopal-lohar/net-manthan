pub mod download_config;

use chrono::{DateTime, Duration, Utc};
use std::path::PathBuf;
use uuid::Uuid;

pub struct Download {
    pub id: Uuid,
    pub url: String,
    pub file: PathBuf,
    pub referrer: Option<String>,
    pub date_added: DateTime<Utc>,
    pub active_time: Duration,
    pub status: DownloadStatus,
    pub parts: DownloadPart,
}

pub enum DownloadStatus {
    Queued,
    Connecting,
    Retrying,
    Downloading,
    Paused,
    Complete,
    Failed,
    Cancelled,
}

pub enum DownloadPart {
    Resumable(Vec<ResumableDownloadPart>),
    NonResumable(NonResumableDownloadPart),
}

pub struct ResumableDownloadPart {
    pub id: Uuid,
    pub status: DownloadStatus,
    pub retry_count: usize,
    pub start_byte: u64,
    pub end_byte: u64,
    pub bytes_downloaded: u64,
    pub current_speed: usize,
}

impl ResumableDownloadPart {
    pub fn get_total_size(&self) -> u64 {
        self.end_byte - self.start_byte + 1
    }
}

pub struct NonResumableDownloadPart {
    pub id: Uuid,
    pub status: DownloadStatus,
    pub retry_count: usize,
    pub total_size: u64,
    pub bytes_downloaded: u64,
    pub current_speed: usize,
}

impl Default for Download {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            url: "http://idk".into(),
            file: "download.exe".into(),
            referrer: None,
            date_added: Utc::now(),
            active_time: Duration::seconds(0),
            status: DownloadStatus::Queued,
            parts: DownloadPart::Resumable(vec![]),
        }
    }
}
