use super::{request::DownloadInfo, status::DownloadStatus};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ChunksInfo {
    InfoLoaded(ChunksInfoLoaded),
    TooManyFileConflicts(String, DownloadInfo),
    ErrorCreatingFile(String, DownloadInfo),
    ErrorLoadingInfo(String),
    Stopped,       // DownloadStatus::Paused (without info loaded)
    Queued,        // DownloadStatus::Queued (without info laoded)
    InfoNotLoaded, // DownloadStatus::Created (info doesn't load at this stage)
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChunksInfoLoaded {
    pub info: DownloadInfo,
    pub parts: DownloadParts,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum DownloadParts {
    Resumable(Vec<ResumablePart>),
    NonResumable(NonResumablePart),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DownloadPart<T: SizeInfo> {
    pub id: i64,
    pub status: DownloadStatus,
    pub error: Option<String>, // error doesn't need to be accessed frequently while status does, so it's stored separately
    pub bytes_downloaded: u64,
    pub current_speed: u64,
    pub size_info: T,
}

impl<T: SizeInfo> DownloadPart<T> {
    pub fn total_size(&self) -> Option<u64> {
        self.size_info.total_size()
    }

    pub fn is_resumable(&self) -> bool {
        self.size_info.is_resumable()
    }

    pub fn apply_header(&self, request: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        self.size_info.apply_headers(request)
    }

    pub fn get_write_head(&self) -> u64 {
        self.size_info.get_start_byte()
            + if self.is_resumable() {
                self.bytes_downloaded
            } else {
                0
            }
    }
}

pub trait SizeInfo {
    fn total_size(&self) -> Option<u64>;
    fn is_resumable(&self) -> bool;
    fn apply_headers(&self, request: reqwest::RequestBuilder) -> reqwest::RequestBuilder;
    fn get_start_byte(&self) -> u64;
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ResumableInfo {
    pub start_byte: u64,
    pub end_byte: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NonResumableInfo {
    // NOTE: we currently do not support downloads with unknown size, but in near future we might
    pub total_size: Option<u64>,
}

impl SizeInfo for ResumableInfo {
    fn total_size(&self) -> Option<u64> {
        Some(self.end_byte - self.start_byte + 1)
    }

    fn is_resumable(&self) -> bool {
        true
    }

    fn apply_headers(&self, request: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        request.header(
            "Range",
            format!("bytes={}-{}", self.get_start_byte(), self.end_byte),
        )
    }

    fn get_start_byte(&self) -> u64 {
        self.start_byte
    }
}

impl SizeInfo for NonResumableInfo {
    fn total_size(&self) -> Option<u64> {
        self.total_size
    }

    fn is_resumable(&self) -> bool {
        false
    }

    fn apply_headers(&self, request: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        request
    }

    fn get_start_byte(&self) -> u64 {
        0
    }
}

pub type ResumablePart = DownloadPart<ResumableInfo>;
pub type NonResumablePart = DownloadPart<NonResumableInfo>;
