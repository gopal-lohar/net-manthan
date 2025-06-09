use crate::download_config::DownloadConfig;
use crate::errors::DownloadError;
use crate::types::DownloadRequest;
use crate::utils::{calculate_chunks, extract_filename};
use crate::{NonResumableDownloadPart, ResumableDownloadPart};
use crate::{
    download_part::{DownloadParts, DownloadPartsProgress},
    types::DownloadStatus,
    utils::format_bytes,
};
use chrono::{DateTime, Duration, Utc};
use reqwest::{Client, header};
use std::{
    path::PathBuf,
    sync::{Arc, atomic::AtomicBool},
};
use tokio::sync::Mutex;
use tracing::info;
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct Download {
    /// Unique identifier for the download.
    pub id: Uuid,
    /// URL of the download.
    pub url: String,
    /// File path where the download will be saved.
    pub file: PathBuf,
    /// file name for customization (comes from request and useless after load_download_info)
    pub file_name: Option<PathBuf>,
    /// Headers for the download.
    pub headers: Option<Vec<String>>,
    /// Referrer URL for the download.
    pub referrer: Option<String>,
    /// chrono DateTime when the download was added.
    pub date_added: DateTime<Utc>,
    /// Duration of active time for the download.
    pub active_time: Duration,
    /// last time the activetime was updated, None means download wasn't active
    pub last_update_time: Option<DateTime<Utc>>,
    /// Current status of the download (Remove it asap and account for it everywhere)
    pub status: DownloadStatus,
    /// Arc Bool token for pausing the download.
    pub stop_token: Arc<AtomicBool>,
    /// configuration options for the download.
    pub config: DownloadConfig,
    /// Parts of the download.
    pub parts: DownloadParts,
    /// Progress of each part, shared using Arc Mutex
    pub progress: DownloadPartsProgress,
}

impl Download {
    pub fn new(request: DownloadRequest, config: &DownloadConfig) -> Self {
        let id = Uuid::new_v4();

        // TODO: maybe add a check for the url validity

        Self {
            id,
            url: request.url,
            file: request.file_dir,
            file_name: request.file_name,
            headers: request.headers,
            referrer: request.referrer,
            date_added: Utc::now(),
            active_time: Duration::zero(),
            last_update_time: None,
            status: DownloadStatus::Created,
            stop_token: Arc::new(AtomicBool::new(false)),
            config: config.clone(),
            parts: DownloadParts::None,
            progress: DownloadPartsProgress::None,
        }
    }

    pub async fn load_download_info(&mut self) -> Result<(), DownloadError> {
        info!("Loading download_info for {:?}", self.id);
        self.status = DownloadStatus::Connecting;
        let client = Client::new();
        let response = match client.get(&self.url).send().await {
            Ok(response) => response,
            Err(err) => {
                self.status = DownloadStatus::Failed;
                return Err(DownloadError::HttpRequestError(err));
            }
        };

        let total_size = response
            .headers()
            .get(header::CONTENT_LENGTH)
            .and_then(|val| val.to_str().ok())
            .and_then(|val| val.parse::<u64>().ok())
            .unwrap_or(0);

        let resume = response.headers().get(header::ACCEPT_RANGES).is_some();

        // if the request doesn't provide a filename, we try to get it from
        // the response headers
        // or from the URL
        // or a fallback name using the download ID
        let mut file = match &self.file_name {
            Some(name) => name.into(),
            None => match extract_filename(response.headers(), &self.url) {
                Some(name) => name,
                None => format!("net-manthan-download-{}", self.id).into(),
            },
        };

        let new_extension = match file.extension() {
            Some(ext) => {
                let ext_str = ext.to_str().expect("Non-UTF8 extension");
                format!("{}.nm", ext_str) // Append ".nm"
            }
            None => String::from("nm"), // No existing extension; use "nm"
        };
        file.set_extension(new_extension);

        self.file_name = Some(file.clone());
        self.file.push(file);

        self.parts = if resume {
            DownloadParts::Resumable(
                calculate_chunks(total_size, self.config.connections_per_server as u64)
                    .iter()
                    .map(|(start_byte, end_byte)| ResumableDownloadPart {
                        id: Uuid::new_v4(),
                        status: DownloadStatus::Queued,
                        start_byte: *start_byte,
                        end_byte: *end_byte,
                        bytes_downloaded: 0,
                        current_speed: 0,
                    })
                    .collect(),
            )
        } else {
            DownloadParts::NonResumable(NonResumableDownloadPart {
                id: Uuid::new_v4(),
                status: DownloadStatus::Queued,
                total_size,
                bytes_downloaded: 0,
                current_speed: 0,
            })
        };

        self.progress = match &self.parts {
            DownloadParts::NonResumable(part) => {
                DownloadPartsProgress::NonResumable(Arc::new(Mutex::new(part.clone())))
            }
            DownloadParts::Resumable(parts) => DownloadPartsProgress::Resumable(
                parts
                    .iter()
                    .map(|part| Arc::new(Mutex::new(part.clone())))
                    .collect(),
            ),
            DownloadParts::None => DownloadPartsProgress::None,
        };

        self.status = DownloadStatus::Queued;

        if total_size != self.get_total_size() {
            self.status = DownloadStatus::Failed;
            self.parts = DownloadParts::None;
            self.progress = DownloadPartsProgress::None;
            return Err(DownloadError::GeneralError("Mismatch in total size".into()));
        }

        Ok(())
    }

    pub fn set_status(&mut self, status: DownloadStatus) {
        match &mut self.parts {
            DownloadParts::NonResumable(part) => part.status = status,
            DownloadParts::Resumable(parts) => {
                for part in parts {
                    part.status = status.clone();
                }
            }
            DownloadParts::None => {}
        }
    }

    pub fn get_total_size(&self) -> u64 {
        match &self.parts {
            DownloadParts::Resumable(parts) => parts.iter().map(|part| part.get_total_size()).sum(),
            DownloadParts::NonResumable(part) => part.total_size,
            DownloadParts::None => 0,
        }
    }

    pub fn get_bytes_downloaded(&self) -> u64 {
        match &self.parts {
            DownloadParts::Resumable(parts) => parts.iter().map(|part| part.bytes_downloaded).sum(),
            DownloadParts::NonResumable(part) => part.bytes_downloaded,
            DownloadParts::None => 0,
        }
    }

    pub fn get_current_speed(&self) -> usize {
        match &self.parts {
            DownloadParts::Resumable(parts) => parts.iter().map(|part| part.current_speed).sum(),
            DownloadParts::NonResumable(part) => part.current_speed,
            DownloadParts::None => 0,
        }
    }

    pub fn get_progress_percentage(&self) -> f64 {
        let total_size = self.get_total_size();
        if total_size == 0 {
            return 0.0;
        }

        let bytes_downloaded = self.get_bytes_downloaded();
        (bytes_downloaded as f64 / total_size as f64) * 100.0
    }

    pub fn get_average_speed(&self) -> usize {
        let milli_seconds = self.active_time.num_milliseconds();
        let milli_seconds = if milli_seconds < 0 {
            0 as u64
        } else {
            milli_seconds as u64
        };
        // If active_time is zero, return current speed instead
        // also a safeguard against division by zero
        if milli_seconds <= 0 {
            return self.get_current_speed();
        }

        let bytes_downloaded = self.get_bytes_downloaded();
        ((bytes_downloaded * 1000) / milli_seconds) as usize
    }

    /// Get a formatted string representation of the average speed
    pub fn get_formatted_average_speed(&self) -> String {
        format!("{}/s", format_bytes(self.get_average_speed() as u64)).into()
    }

    /// Get a formatted string representation of the current speed
    pub fn get_formatted_current_speed(&self) -> String {
        format!("{}/s", format_bytes(self.get_current_speed() as u64)).into()
    }

    pub async fn update_progress(&mut self) {
        match &self.progress {
            DownloadPartsProgress::Resumable(progress_parts) => {
                let mut updated_parts = Vec::with_capacity(progress_parts.len());

                for part in progress_parts {
                    let locked_part = part.lock().await;
                    updated_parts.push(locked_part.clone());
                }

                self.parts = DownloadParts::Resumable(updated_parts);
            }
            DownloadPartsProgress::NonResumable(progress_part) => {
                let locked_part = progress_part.lock().await;
                self.parts = DownloadParts::NonResumable(locked_part.clone());
            }
            DownloadPartsProgress::None => {
                self.parts = DownloadParts::None;
            }
        }

        // if we are  not actively downloading, selt last_update_time to none
        if match self.get_status() {
            DownloadStatus::Connecting => false,
            DownloadStatus::Retrying => false,
            DownloadStatus::Downloading => false,
            _ => true,
        } {
            self.last_update_time = None;
        };

        match self.last_update_time {
            Some(last_update_time) => {
                let now = Utc::now();
                let diff = now - last_update_time;
                self.last_update_time = Some(now);
                self.active_time += diff;
            }
            None => {}
        }
    }

    pub fn get_status(&self) -> DownloadStatus {
        match &self.parts {
            DownloadParts::Resumable(parts) => {
                Download::calculate_status(parts.iter().map(|p| p.status.clone()).collect())
            }
            DownloadParts::NonResumable(part) => part.status.clone(),
            DownloadParts::None => DownloadStatus::Created,
        }
    }

    pub fn calculate_status(status_vec: Vec<DownloadStatus>) -> DownloadStatus {
        // If there are no parts, return the current status
        if status_vec.is_empty() {
            return DownloadStatus::Created;
        }

        // Check if all parts are complete
        let all_complete = status_vec
            .iter()
            .all(|p| matches!(p, DownloadStatus::Complete));
        if all_complete {
            return DownloadStatus::Complete;
        }

        // Check if all parts are Queued
        let all_queued = status_vec
            .iter()
            .all(|p| matches!(p, DownloadStatus::Queued));
        if all_queued {
            return DownloadStatus::Queued;
        }

        // Check if all parts are Cancelled
        let all_cancelled = status_vec
            .iter()
            .all(|p| matches!(p, DownloadStatus::Cancelled));
        if all_cancelled {
            return DownloadStatus::Cancelled;
        }

        // Check if any part is downloading
        let any_downloading = status_vec
            .iter()
            .any(|p| matches!(p, DownloadStatus::Downloading));
        if any_downloading {
            return DownloadStatus::Downloading;
        }

        // Check if all non-complete parts are connecting
        let all_remaining_connecting = status_vec
            .iter()
            .filter(|p| !matches!(p, DownloadStatus::Complete))
            .all(|p| matches!(p, DownloadStatus::Connecting));
        if all_remaining_connecting {
            return DownloadStatus::Connecting;
        }

        // Check if all non-complete parts are retrying
        let all_remaining_retrying = status_vec
            .iter()
            .filter(|p| !matches!(p, DownloadStatus::Complete))
            .all(|p| matches!(p, DownloadStatus::Retrying));
        if all_remaining_retrying {
            return DownloadStatus::Retrying;
        }

        // Check if all non-complete parts are failed
        let all_remaining_failed = status_vec
            .iter()
            .filter(|p| !matches!(p, DownloadStatus::Complete))
            .all(|p| matches!(p, DownloadStatus::Failed));
        if all_remaining_failed {
            return DownloadStatus::Failed;
        }

        // Check if all parts are paused
        let all_paused = status_vec
            .iter()
            .all(|p| matches!(p, DownloadStatus::Paused));
        if all_paused {
            return DownloadStatus::Paused;
        }

        // Default to Created
        DownloadStatus::Created
    }
}
