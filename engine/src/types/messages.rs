use super::{
    others::DownloadConfig,
    request::{DownloadInfo, DownloadRequest},
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DownloadRequestMessage {
    pub request: DownloadRequest,
    pub config: Option<DownloadConfig>,
    pub info: Option<DownloadInfo>,
}
