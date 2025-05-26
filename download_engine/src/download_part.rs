use std::sync::Arc;

use crate::types::DownloadStatus;
use tokio::sync::Mutex;
use uuid::Uuid;

#[derive(Clone, Debug)]
pub enum DownloadPart {
    Resumable(ResumableDownloadPart),
    NonResumable(NonResumableDownloadPart),
}

#[derive(Clone, Debug)]
pub struct ResumableDownloadPart {
    pub common: Common,
    pub range: ByteRange,
}

#[derive(Clone, Debug)]
pub struct NonResumableDownloadPart {
    pub common: Common,
    pub size: u64,
}

#[derive(Clone, Debug)]
pub struct Common {
    pub id: Uuid,
    pub status: DownloadStatus,
    pub bytes_downloaded: u64,
    pub current_speed: u64,
    pub progress: Arc<Mutex<PartProgress>>,
}

#[derive(Clone, Debug)]
pub struct PartProgress {
    pub status: DownloadStatus,
    pub bytes_downloaded: u64,
    pub current_speed: u64,
}

#[derive(Clone, Debug)]
pub struct ByteRange {
    pub start_byte: u64,
    pub end_byte: u64,
}

// Deref to access common fields directly
impl std::ops::Deref for DownloadPart {
    type Target = Common;

    fn deref(&self) -> &Self::Target {
        match self {
            DownloadPart::Resumable(part) => &part.common,
            DownloadPart::NonResumable(part) => &part.common,
        }
    }
}

impl std::ops::DerefMut for DownloadPart {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            DownloadPart::Resumable(part) => &mut part.common,
            DownloadPart::NonResumable(part) => &mut part.common,
        }
    }
}

// get_total_size
impl NonResumableDownloadPart {
    pub fn get_total_size(&self) -> u64 {
        self.size
    }
}

impl ResumableDownloadPart {
    pub fn get_total_size(&self) -> u64 {
        self.range.end_byte - self.range.start_byte + 1
    }
}

impl DownloadPart {
    pub fn get_total_size(&self) -> u64 {
        match self {
            DownloadPart::Resumable(part) => part.get_total_size(),
            DownloadPart::NonResumable(part) => part.get_total_size(),
        }
    }
}
