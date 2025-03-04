use std::path::PathBuf;

use crate::{config::NetManthanConfig, Download};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DownloadRequest {
    pub url: String,
    pub referrer: Option<String>,
    pub filepath: Option<PathBuf>,
    pub filename: Option<String>,
    pub headers: Option<Vec<String>>,
}

/// for communicating progress for each part of a download
/// (the aggregator thread just sends a vector of these)
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct PartProgress {
    pub part_id: String,
    pub bytes_downloaded: u64,
    pub total_bytes: u64,
    pub speed: u64,
    pub timestamp: DateTime<Utc>,
    pub error: bool,
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
    DownloadsList(Vec<Download>),
    Config(NetManthanConfig),
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
