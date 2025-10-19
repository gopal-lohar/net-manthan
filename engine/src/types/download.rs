use super::{
    chunks::{
        ChunksInfo, ChunksInfoLoaded, DownloadParts, NonResumableInfo, NonResumablePart,
        ResumableInfo, ResumablePart, SizeInfo,
    },
    errors::DownloadError,
    messages::DownloadRequestMessage,
    others::{DownloadConfig, TimeStamps},
    request::{DownloadInfo, DownloadRequest},
    status::DownloadStatus,
};
use crate::helpers::random::generate_random_id;
use chrono::Duration;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tracing::{info, trace, warn};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Download {
    pub id: i64,
    pub request: DownloadRequest,
    pub time_stamps: TimeStamps,
    pub config: DownloadConfig,
    pub chunks_info: ChunksInfo,
}

impl Download {
    pub async fn new(request: DownloadRequestMessage) -> Self {
        let mut download = Download {
            id: generate_random_id(),
            request: request.request,
            time_stamps: TimeStamps::now(),
            config: request.config.unwrap_or_default(),
            chunks_info: ChunksInfo::InfoNotLoaded,
        };
        if let Some(info) = request.info {
            download.set_info(info).await;
        }
        download
    }

    pub fn has_info_loaded(&self) -> bool {
        match self.chunks_info {
            ChunksInfo::InfoLoaded(_) => true,
            ChunksInfo::TooManyFileConflicts(_, _) => true,
            ChunksInfo::ErrorCreatingFile(_, _) => true,
            ChunksInfo::ErrorLoadingInfo(_) => false,
            ChunksInfo::Stopped => false,
            ChunksInfo::Queued => false,
            ChunksInfo::InfoNotLoaded => false,
        }
    }

    pub fn get_filename(&self) -> Option<String> {
        match &self.chunks_info {
            ChunksInfo::InfoLoaded(chunk_info) => Some(chunk_info.info.file_name.clone()),
            ChunksInfo::TooManyFileConflicts(_, info) => Some(info.file_name.clone()),
            ChunksInfo::ErrorCreatingFile(_, info) => Some(info.file_name.clone()),
            _ => self.request.rename.clone(),
        }
    }

    /// returns (download status, in Failed use get_errors to get the errors)
    pub fn get_status(&self) -> DownloadStatus {
        match &self.chunks_info {
            ChunksInfo::InfoNotLoaded => DownloadStatus::Created,
            ChunksInfo::Queued => DownloadStatus::Queued,
            ChunksInfo::Stopped => DownloadStatus::Paused,
            ChunksInfo::ErrorCreatingFile(_, _) => DownloadStatus::Failed,
            ChunksInfo::ErrorLoadingInfo(_) => DownloadStatus::Failed,
            ChunksInfo::TooManyFileConflicts(_, _) => DownloadStatus::Failed,
            ChunksInfo::InfoLoaded(loaded) => match &loaded.parts {
                DownloadParts::NonResumable(part) => part.status.clone(),
                DownloadParts::Resumable(parts) => {
                    Self::aggregate_parts_status(parts).unwrap_or(DownloadStatus::Failed)
                }
            },
        }
    }

    /// if info has loaded then set status for all parts, else do nothing. if info hasn't loaded then please handle it in self.chunks_info yourselves
    pub fn set_status(&mut self, status: DownloadStatus) {
        if self.has_info_loaded() {
            if let ChunksInfo::InfoLoaded(loaded) = &mut self.chunks_info {
                match &mut loaded.parts {
                    DownloadParts::NonResumable(part) => {
                        part.status = status;
                    }
                    DownloadParts::Resumable(parts) => {
                        for part in parts {
                            part.status = status.clone();
                        }
                    }
                }
            }
        }
    }

    fn aggregate_parts_status(parts: &[ResumablePart]) -> Option<DownloadStatus> {
        // Created: Either all or none
        // Queued: Either all or none
        // Connecting: when downloading
        // Retrying: when downloading
        // Downloading: when downloading
        // Paused: when pausing
        // Complete: when downloading or pausing
        // Failed: when downloading or pausing
        // Cancelled: all or none

        // if all are same, return it
        let first = parts.first()?;
        if parts.iter().all(|s| s.status == first.status) {
            return Some(first.status.clone());
        }

        // if any part is downloading, return downloading
        if parts
            .iter()
            .any(|s| matches!(s.status, DownloadStatus::Downloading))
        {
            return Some(DownloadStatus::Downloading);
        }

        let non_complete: Vec<&ResumablePart> = parts
            .iter()
            .filter(|p| !matches!(p.status, DownloadStatus::Complete))
            .collect();
        let first_non_complete: &ResumablePart = non_complete.first()?;

        // if all parts other than complete are same then return it
        if non_complete
            .iter()
            .all(|s| s.status == first_non_complete.status)
        {
            return Some(first_non_complete.status.clone());
        };

        let priorities = vec![
            DownloadStatus::Connecting,
            DownloadStatus::Retrying,
            DownloadStatus::Paused,
            DownloadStatus::Failed,
            DownloadStatus::Cancelled,
            DownloadStatus::Queued,
            DownloadStatus::Created,
        ];

        priorities
            .into_iter()
            .find(|priority| parts.iter().any(|s| &s.status == priority))
    }

    pub async fn set_info(&mut self, mut info: DownloadInfo) {
        let final_file_name =
            resolve_file_name_conflict(&self.request.directory, &info.file_name).await;
        self.chunks_info = match final_file_name {
            Some(name) => {
                info.file_name = name.clone();
                let file_path = self.request.get_file_path(&info);
                match tokio::fs::File::create(file_path).await {
                    Ok(_) => ChunksInfo::InfoLoaded(ChunksInfoLoaded {
                        parts: match &info.size {
                            Some(size) => {
                                trace!("Download size for {} is {}", name, size);
                                let size = *size;
                                if size == 0 {
                                    DownloadParts::NonResumable(NonResumablePart {
                                        id: generate_random_id(),
                                        status: DownloadStatus::Created,
                                        error: None,
                                        bytes_downloaded: 0,
                                        current_speed: 0,
                                        size_info: NonResumableInfo {
                                            total_size: info.size,
                                        },
                                    })
                                } else {
                                    DownloadParts::Resumable(
                                        calculate_chunks(
                                            size,
                                            self.config.connections_per_server as u64,
                                        )
                                        .iter()
                                        .map(|c| ResumablePart {
                                            id: generate_random_id(),
                                            status: DownloadStatus::Created,
                                            error: None,
                                            bytes_downloaded: 0,
                                            current_speed: 0,
                                            size_info: ResumableInfo {
                                                start_byte: c.0,
                                                end_byte: c.1,
                                            },
                                        })
                                        .collect(),
                                    )
                                }
                            }
                            None => DownloadParts::NonResumable(NonResumablePart {
                                id: generate_random_id(),
                                status: DownloadStatus::Created,
                                error: None,
                                bytes_downloaded: 0,
                                current_speed: 0,
                                size_info: NonResumableInfo {
                                    total_size: info.size,
                                },
                            }),
                        },
                        info,
                    }),
                    Err(err) => {
                        warn!("Error creating file: {}", err);
                        ChunksInfo::ErrorCreatingFile(
                            DownloadError::FileSystemError(err).to_string(),
                            info,
                        )
                    }
                }
            }
            None => {
                warn!("Too many file conflicts");
                ChunksInfo::TooManyFileConflicts(
                    DownloadError::TooManyFileConflicts.to_string(),
                    info,
                )
            }
        };
    }

    // Sum of speed of all parts
    pub fn total_speed(&self) -> u64 {
        match &self.chunks_info {
            ChunksInfo::InfoLoaded(loaded) => match &loaded.parts {
                DownloadParts::Resumable(parts) => parts.iter().map(|p| p.current_speed).sum(),
                DownloadParts::NonResumable(part) => part.current_speed,
            },
            _ => 0,
        }
    }

    // Total size of the download
    pub fn total_size(&self) -> Option<u64> {
        match &self.chunks_info {
            ChunksInfo::InfoLoaded(loaded) => match &loaded.parts {
                DownloadParts::Resumable(parts) => {
                    Some(parts.iter().filter_map(|p| p.size_info.total_size()).sum())
                }
                DownloadParts::NonResumable(part) => part.size_info.total_size(),
            },
            _ => None,
        }
    }

    // Total bytes downloaded
    pub fn bytes_downloaded(&self) -> u64 {
        match &self.chunks_info {
            ChunksInfo::InfoLoaded(loaded) => match &loaded.parts {
                DownloadParts::Resumable(parts) => parts.iter().map(|p| p.bytes_downloaded).sum(),
                DownloadParts::NonResumable(part) => part.bytes_downloaded,
            },
            _ => 0,
        }
    }

    // Estimated time remaining
    pub fn estimated_time_remaining(&self) -> Option<Duration> {
        let total_size = self.total_size()?;
        let downloaded = self.bytes_downloaded();
        let remaining = total_size.checked_sub(downloaded)?;
        let speed = self.total_speed();

        if speed == 0 || remaining == 0 {
            return None;
        }

        let seconds = (remaining as f64) / (speed as f64);
        Some(Duration::milliseconds((seconds * 1000.0) as i64))
    }

    pub fn progress_percentage(&self) -> Option<f64> {
        let downloaded = self.bytes_downloaded();
        let total = self.total_size()?;

        if total == 0 {
            return None;
        }

        Some((downloaded as f64 / total as f64) * 100.0)
    }

    pub fn is_resumable(&self) -> bool {
        match &self.chunks_info {
            ChunksInfo::InfoLoaded(loaded) => match &loaded.parts {
                DownloadParts::Resumable(_) => true,
                DownloadParts::NonResumable(_) => false,
            },
            _ => false,
        }
    }

    pub fn average_speed(&self) -> u64 {
        if self.time_stamps.active_time.as_seconds_f64() == 0. {
            0
        } else {
            ((self.bytes_downloaded() as f64) / self.time_stamps.active_time.as_seconds_f64())
                as u64
        }
    }
}

/// split total size into chunks with ~equal size
fn calculate_chunks(total_size: u64, num_chunks: u64) -> Vec<(u64, u64)> {
    info!("Calculating CHUNKS");
    // if total_size < num_chunks {
    // vec![(0, total_size - 1)]
    // } else {
    let base_chunk_size = total_size / num_chunks;
    let remainder = total_size % num_chunks;
    (0..num_chunks)
        .map(|i| {
            let start = i * base_chunk_size + u64::min(i, remainder);
            let end = if i < remainder {
                start + base_chunk_size // Extra byte for first `remainder` chunks
            } else {
                start + base_chunk_size - 1 // No extra byte
            };
            // Cap at total_size - 1 to avoid overshooting
            (start, end.min(total_size.saturating_sub(1)))
        })
        .collect()
    // }
}

async fn resolve_file_name_conflict(dir: &str, original_name: &str) -> Option<String> {
    let mut path = PathBuf::from(&dir);
    path.push(original_name);

    // If file doesn't exist, use original name
    if !path.exists() {
        return Some(original_name.to_string());
    }

    // Extract file stem and extension
    let random_name = DownloadRequest::get_random_file_name().to_owned();
    let file_stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(&random_name);
    let extension = path
        .extension()
        .and_then(|s| s.to_str())
        .map(|s| format!(".{s}"))
        .unwrap_or_default();

    // Find available numbered filename
    let mut counter = 1;
    loop {
        let new_name = format!("{file_stem} ({counter}){extension}");
        let mut new_path = PathBuf::from(&dir);
        new_path.push(&new_name);

        if !new_path.exists() {
            return Some(new_name);
        }
        counter += 1;
        // Prevent infinite loop with a reasonable limit
        if counter > 9999 {
            return None;
        }
    }
}
