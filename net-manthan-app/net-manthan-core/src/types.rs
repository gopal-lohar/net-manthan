use std::path::PathBuf;

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DownloadRequest {
    pub url: String,
    pub filepath: PathBuf,
    pub headers: Option<Vec<String>>,
    pub parts: Option<Vec<DownloadPart>>,
    pub config: DownloadRequestConfig,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DownloadRequestConfig {
    pub thread_count: u8,
    pub buffer_size: usize,
    pub update_interval: Duration,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DownloadPart {
    pub part_id: u8,
    pub bytes_downloaded: u64,
    pub range: Option<(u64, u64)>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct PartProgress {
    pub part_id: u8,
    pub bytes_downloaded: u64,
    pub total_bytes: u64,
    pub speed: u64,
    pub timestamp: DateTime<Utc>,
    pub error: bool,
}

// ----------------- Config -----------------
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NetManthanConfig {
    pub auto_resume: bool,
    pub default_threads: u8,
    pub single_threaded_buffer_size: u64,
    pub multi_threaded_buffer_size: u64,
    pub download_dir: PathBuf,
    pub database_path: PathBuf,
}

// ----------------- IPC MESSAGES -----------------
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum IpcRequest {
    HeartBeat,
    ListDownloads {
        incomplete_only: bool,
        detailed: bool,
        limit: Option<usize>,
    },
    StartDownload {
        url: String,
        output_path: Option<PathBuf>,
        thread_count: Option<u8>,
        headers: Option<Vec<String>>,
    },
    ResumeDownloads {
        ids: Vec<u64>, // Empty means all
    },
    PauseDownloads {
        ids: Vec<u64>, // Empty means all
    },
    RemoveDownloads {
        ids: Vec<u64>,
        delete_files: bool,
    },
    UpdateDownload {
        id: u64,
        new_url: Option<String>,
        new_output_path: Option<PathBuf>,
    },
    WatchDownloads {
        ids: Vec<u64>, // Empty means all
        interval_ms: u64,
        detailed: bool,
    },
    GetConfig,
    SetConfig(NetManthanConfig),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum IpcResponse {
    HeartBeat,
    Success,
    Error(String),
    DownloadProgress(DownloadProgress),
    DownloadsList(Vec<DownloadInfo>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadInfo {
    pub id: u64,
    pub url: String,
    pub output_path: PathBuf,
    pub status: DownloadStatus,
    pub created_at: DateTime<Utc>,
    pub total_size: Option<u64>,
    pub downloaded_size: u64,
    pub thread_count: u8,
    pub average_speed: f64, // bytes per second
    pub error: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum DownloadStatus {
    Queued,
    Connecting,
    Downloading,
    Paused,
    Completed,
    Error,
}

/// Real-time progress information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadProgress {
    pub id: u64,
    pub status: DownloadStatus,
    pub downloaded_size: u64,
    pub total_size: Option<u64>,
    pub speed: u64,
    pub threads_info: Option<Vec<ThreadProgress>>,
}

/// Per-thread progress information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadProgress {
    pub thread_id: u8,
    pub start_byte: u64,
    pub end_byte: u64,
    pub current_byte: u64,
    pub speed: f64,
    pub status: DownloadStatus,
}

impl DownloadProgress {
    pub fn progress_percentage(&self) -> Option<f64> {
        self.total_size
            .map(|total| (self.downloaded_size as f64 / total as f64) * 100.0)
    }
}
