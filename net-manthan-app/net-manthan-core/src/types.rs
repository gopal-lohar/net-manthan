use std::{collections::HashMap, path::PathBuf};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// Shared message type between client and server
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Message {
    HeartBeat,
    DownloadRequest(DownloadRequest),
    DownnloadResponse(String),
    InvalidMessage,
    ProgressRequest(Vec<u64>),
    ProgressResponse(HashMap<u32, ChunkProgress>),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DownloadRequest {
    pub url: String,
    pub filename: String,
    pub mime: Option<String>,
    pub referrer: Option<String>,
    pub headers: Option<Vec<String>>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ChunkProgress {
    pub download_id: u64,
    pub chunk_id: u32,
    pub bytes_downloaded: u64,
    pub total_bytes: u64,
    pub speed: f64,
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
pub enum IpcMessage {
    Request(IpcRequest),
    Response(IpcResponse),
    HeartBeat,
    InvalidMessage,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum IpcRequest {
    ListDownloads {
        incomplete_only: bool,
        detailed: bool,
        limit: Option<usize>,
    },
    StartDownload {
        url: String,
        output_path: Option<PathBuf>,
        thread_count: Option<u8>,
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
    Success,
    Error(String),

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
    pub speed: f64,
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
