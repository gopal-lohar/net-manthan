
use serde::{Deserialize, Serialize};

// Shared message type between client and server
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Message {
    HeartBeat,
    DownloadRequest(DownloadRequest),
    DownnloadResponse(String),
    InvalidMessage,
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DownloadRequest {
    pub url: String,
    pub filename: String,
    pub mime: Option<String>,
    pub referrer: Option<String>,
    pub headers: Option<Vec<String>>,
}

