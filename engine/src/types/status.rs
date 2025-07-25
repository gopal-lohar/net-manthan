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
