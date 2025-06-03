use std::sync::Arc;

use crate::types::DownloadStatus;
use tokio::sync::Mutex;
use uuid::Uuid;

#[derive(Clone, Debug)]
struct PartProgress {
    pub status: DownloadStatus,
    pub bytes_downloaded: u64,
    pub current_speed: u64,
}

#[derive(Clone, Debug)]
pub struct DownloadPart {
    pub id: Uuid,
    pub status: DownloadStatus,
    pub bytes_downloaded: u64,
    pub current_speed: u64,
    progress: Arc<Mutex<PartProgress>>,
}

#[derive(Clone, Debug)]
pub struct ByteRange {
    pub start_byte: u64,
    pub end_byte: u64,
}

impl ByteRange {
    pub fn get_total_size(&self) -> u64 {
        self.end_byte - self.start_byte + 1
    }
}

// Trait for getting total size - implemented differently for each type
pub trait TotalSize {
    fn get_total_size(&self) -> u64;
}

// Type-safe wrappers for different part types
#[derive(Clone, Debug)]
pub struct ResumableDownloadPart {
    pub part: DownloadPart,
    pub range: ByteRange,
}

impl TotalSize for ResumableDownloadPart {
    fn get_total_size(&self) -> u64 {
        self.range.get_total_size()
    }
}

// Deref to access common fields directly
impl std::ops::Deref for ResumableDownloadPart {
    type Target = DownloadPart;

    fn deref(&self) -> &Self::Target {
        &self.part
    }
}

impl std::ops::DerefMut for ResumableDownloadPart {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.part
    }
}

#[derive(Clone, Debug)]
pub struct NonResumableDownloadPart {
    pub part: DownloadPart,
    pub total_size: u64,
}

impl TotalSize for NonResumableDownloadPart {
    fn get_total_size(&self) -> u64 {
        self.total_size
    }
}

// Deref to access common fields directly
impl std::ops::Deref for NonResumableDownloadPart {
    type Target = DownloadPart;

    fn deref(&self) -> &Self::Target {
        &self.part
    }
}

impl std::ops::DerefMut for NonResumableDownloadPart {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.part
    }
}

#[derive(Clone, Debug)]
pub enum DownloadParts {
    Resumable(Vec<ResumableDownloadPart>),
    NonResumable(NonResumableDownloadPart),
    None,
}

impl DownloadParts {
    pub fn get_parts(&self) -> Vec<&DownloadPart> {
        match self {
            DownloadParts::Resumable(parts) => parts.iter().map(|p| &p.part).collect(),
            DownloadParts::NonResumable(part) => vec![&part.part],
            DownloadParts::None => vec![],
        }
    }

    pub fn get_parts_mut(&mut self) -> Vec<&mut DownloadPart> {
        match self {
            DownloadParts::Resumable(parts) => parts.iter_mut().map(|p| &mut p.part).collect(),
            DownloadParts::NonResumable(part) => vec![&mut part.part],
            DownloadParts::None => vec![],
        }
    }

    pub fn get_total_size(&self) -> u64 {
        match self {
            DownloadParts::Resumable(parts) => parts.iter().map(|p| p.get_total_size()).sum(),
            DownloadParts::NonResumable(part) => part.get_total_size(),
            DownloadParts::None => 0,
        }
    }

    pub fn is_resumable(&self) -> bool {
        matches!(self, DownloadParts::Resumable(_))
    }

    pub fn total_bytes_downloaded(&self) -> u64 {
        self.get_parts().iter().map(|p| p.bytes_downloaded).sum()
    }

    pub fn set_status(&mut self, status: DownloadStatus) {
        for part in self.get_parts_mut() {
            part.status = status.clone();
        }
    }

    pub async fn update_progress(&mut self) {
        for part in self.get_parts_mut() {
            let progress = part.progress.lock().await;
            part.status = progress.status.clone();
            part.bytes_downloaded = progress.bytes_downloaded;
            part.current_speed = progress.current_speed;
        }
    }
}
