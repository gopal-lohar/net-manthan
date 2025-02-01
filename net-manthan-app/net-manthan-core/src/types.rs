use std::collections::HashMap;

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
