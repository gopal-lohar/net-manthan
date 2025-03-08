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
    pub speed_in_bytes: u64,
    pub status: DownloadStatus,
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
    GetActiveDownloads {},
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
    DownloadsList(Vec<Download>),
    Config(NetManthanConfig),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum DownloadStatus {
    Queued,
    Connecting,
    Downloading,
    Paused,
    Completed(DateTime<Utc>),
    Failed(String),
    Cancelled,
}
