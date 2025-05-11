use std::sync::Arc;

use crate::types::DownloadStatus;
use tokio::sync::Mutex;
use uuid::Uuid;

#[derive(Clone, Debug)]
pub enum DownloadParts {
    Resumable(Vec<ResumableDownloadPart>),
    NonResumable(NonResumableDownloadPart),
    None,
}

#[derive(Clone, Debug)]
pub enum DownloadPartsProgress {
    Resumable(Vec<Arc<Mutex<ResumableDownloadPart>>>),
    NonResumable(Arc<Mutex<NonResumableDownloadPart>>),
    None,
}

#[derive(Clone, Debug)]
pub enum DownloadPart {
    Resumable(ResumableDownloadPart),
    NonResumable(NonResumableDownloadPart),
}

#[derive(Clone, Debug)]
pub enum DownloadProgressPart {
    Resumable(Arc<Mutex<ResumableDownloadPart>>),
    NonResumable(Arc<Mutex<NonResumableDownloadPart>>),
}

impl DownloadProgressPart {
    pub async fn update_status(&self, status: DownloadStatus) {
        match self {
            DownloadProgressPart::Resumable(part) => part.lock().await.status = status,
            DownloadProgressPart::NonResumable(part) => part.lock().await.status = status,
        };
    }
}

#[derive(Clone, Debug)]
pub struct ResumableDownloadPart {
    pub id: Uuid,
    pub status: DownloadStatus,
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

#[derive(Clone, Debug)]
pub struct NonResumableDownloadPart {
    pub id: Uuid,
    pub status: DownloadStatus,
    pub total_size: u64,
    pub bytes_downloaded: u64,
    pub current_speed: usize,
}
