use std::path::PathBuf;

#[derive(Clone, Debug)]
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

#[derive(Debug, Clone)]
pub struct DownloadRequest {
    pub url: String,
    pub file_dir: PathBuf,
    pub file_name: Option<PathBuf>,
    pub referrer: Option<String>,
    pub headers: Option<Vec<String>>,
}
