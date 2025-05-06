use crate::types::DownloadRequest;
use crate::{download_part::DownloadPart, types::DownloadStatus, utils::format_speed};
use chrono::{DateTime, Duration, Utc};
use std::{
    path::PathBuf,
    sync::{Arc, atomic::AtomicBool},
};
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct Download {
    /// Unique identifier for the download.
    pub id: Uuid,
    /// URL of the download.
    pub url: String,
    /// File path where the download will be saved.
    pub file: PathBuf,
    /// Referrer URL for the download.
    pub referrer: Option<String>,
    /// chrono DateTime when the download was added.
    pub date_added: DateTime<Utc>,
    /// Duration of active time for the download.
    pub active_time: Duration,
    /// Current status of the download.
    status: DownloadStatus,
    /// Arc Bool token for pausing the download.
    pub stop_token: Arc<AtomicBool>,
    /// Parts of the download.
    pub parts: DownloadPart,
}

impl Download {
    pub fn new(request: DownloadRequest) -> Self {
        let id = Uuid::new_v4();

        Self {
            id,
            url: request.url,
            file: request.file_dir.join(match request.file_name.clone() {
                Some(name) => name,
                None => format!("net-manthan-download-{}.nm", id),
            }),
            referrer: request.referrer,
            date_added: Utc::now(),
            active_time: Duration::zero(),
            status: DownloadStatus::Created,
            stop_token: Arc::new(AtomicBool::new(false)),
            parts: DownloadPart::None,
        }
    }

    pub fn get_total_size(&self) -> u64 {
        match &self.parts {
            DownloadPart::Resumable(parts) => parts.iter().map(|part| part.get_total_size()).sum(),
            DownloadPart::NonResumable(part) => part.total_size,
            DownloadPart::None => 0,
        }
    }

    pub fn get_bytes_downloaded(&self) -> u64 {
        match &self.parts {
            DownloadPart::Resumable(parts) => parts.iter().map(|part| part.bytes_downloaded).sum(),
            DownloadPart::NonResumable(part) => part.bytes_downloaded,
            DownloadPart::None => 0,
        }
    }

    pub fn get_current_speed(&self) -> usize {
        match &self.parts {
            DownloadPart::Resumable(parts) => parts.iter().map(|part| part.current_speed).sum(),
            DownloadPart::NonResumable(part) => part.current_speed,
            DownloadPart::None => 0,
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
        // If active_time is zero, return current speed instead
        if self.active_time.num_seconds() <= 0 {
            return self.get_current_speed();
        }

        // Calculate bytes downloaded per second over the entire active time
        let bytes_downloaded = self.get_bytes_downloaded();
        let seconds = self.active_time.num_seconds() as u64;

        // Safeguard against division by zero
        if seconds == 0 {
            return 0;
        }

        ((bytes_downloaded as f64) / (seconds as f64)) as usize
    }

    /// Get a formatted string representation of the average speed
    pub fn get_formatted_average_speed(&self) -> String {
        format_speed(self.get_average_speed() as u64)
    }

    /// Get a formatted string representation of the current speed
    pub fn get_formatted_current_speed(&self) -> String {
        format_speed(self.get_current_speed() as u64)
    }

    pub fn get_status(&self) -> DownloadStatus {
        match &self.parts {
            DownloadPart::Resumable(parts) => {
                // If there are no parts, return the current status
                if parts.is_empty() {
                    return self.status.clone();
                }

                // Check if all parts are complete
                let all_complete = parts
                    .iter()
                    .all(|p| matches!(p.status, DownloadStatus::Complete));
                if all_complete {
                    return DownloadStatus::Complete;
                }

                // Check if all parts are Queued
                let all_queued = parts
                    .iter()
                    .all(|p| matches!(p.status, DownloadStatus::Queued));
                if all_queued {
                    return DownloadStatus::Queued;
                }

                // Check if all parts are Cancelled
                let all_cancelled = parts
                    .iter()
                    .all(|p| matches!(p.status, DownloadStatus::Cancelled));
                if all_cancelled {
                    return DownloadStatus::Cancelled;
                }

                // Check if any part is downloading
                let any_downloading = parts
                    .iter()
                    .any(|p| matches!(p.status, DownloadStatus::Downloading));
                if any_downloading {
                    return DownloadStatus::Downloading;
                }

                // Check if all non-complete parts are connecting
                let all_remaining_connecting = parts
                    .iter()
                    .filter(|p| !matches!(p.status, DownloadStatus::Complete))
                    .all(|p| matches!(p.status, DownloadStatus::Connecting));
                if all_remaining_connecting {
                    return DownloadStatus::Connecting;
                }

                // Check if all non-complete parts are retrying
                let all_remaining_retrying = parts
                    .iter()
                    .filter(|p| !matches!(p.status, DownloadStatus::Complete))
                    .all(|p| matches!(p.status, DownloadStatus::Retrying));
                if all_remaining_retrying {
                    return DownloadStatus::Retrying;
                }

                // Check if all non-complete parts are failed
                let all_remaining_failed = parts
                    .iter()
                    .filter(|p| !matches!(p.status, DownloadStatus::Complete))
                    .all(|p| matches!(p.status, DownloadStatus::Failed));
                if all_remaining_failed {
                    return DownloadStatus::Failed;
                }

                // Check if all parts are paused
                let all_paused = parts
                    .iter()
                    .all(|p| matches!(p.status, DownloadStatus::Paused));
                if all_paused {
                    return DownloadStatus::Paused;
                }

                // Default to current status if no specific condition is met
                self.status.clone()
            }

            DownloadPart::NonResumable(part) => part.status.clone(),
            DownloadPart::None => DownloadStatus::Created,
        }
    }
}
