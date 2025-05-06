pub mod download_config;

use chrono::{DateTime, Duration, Utc};
use std::path::PathBuf;
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
    pub status: DownloadStatus,
    /// Parts of the download.
    pub parts: DownloadPart,
}

#[derive(Clone, Debug)]
pub enum DownloadStatus {
    Queued,
    Connecting,
    Retrying,
    Downloading,
    Paused,
    Complete,
    Failed,
    Cancelled,
}

#[derive(Clone, Debug)]
pub enum DownloadPart {
    Resumable(Vec<ResumableDownloadPart>),
    NonResumable(NonResumableDownloadPart),
}

#[derive(Clone, Debug)]
pub struct ResumableDownloadPart {
    pub id: Uuid,
    pub status: DownloadStatus,
    pub retry_count: usize,
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
    pub retry_count: usize,
    pub total_size: u64,
    pub bytes_downloaded: u64,
    pub current_speed: usize,
}

impl Default for Download {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            url: "http://idk".into(),
            file: "download.exe".into(),
            referrer: None,
            date_added: Utc::now(),
            active_time: Duration::seconds(0),
            status: DownloadStatus::Queued,
            parts: DownloadPart::Resumable(vec![]),
        }
    }
}

impl Download {
    pub fn get_total_size(&self) -> u64 {
        match &self.parts {
            DownloadPart::Resumable(parts) => parts.iter().map(|part| part.get_total_size()).sum(),
            DownloadPart::NonResumable(part) => part.total_size,
        }
    }

    pub fn get_bytes_downloaded(&self) -> u64 {
        match &self.parts {
            DownloadPart::Resumable(parts) => parts.iter().map(|part| part.bytes_downloaded).sum(),
            DownloadPart::NonResumable(part) => part.bytes_downloaded,
        }
    }

    pub fn get_current_speed(&self) -> usize {
        match &self.parts {
            DownloadPart::Resumable(parts) => parts.iter().map(|part| part.current_speed).sum(),
            DownloadPart::NonResumable(part) => part.current_speed,
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
        }
    }
}

fn format_speed(bytes_per_second: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes_per_second < KB {
        format!("{} B/s", bytes_per_second)
    } else if bytes_per_second < MB {
        format!("{:.2} KB/s", bytes_per_second as f64 / KB as f64)
    } else if bytes_per_second < GB {
        format!("{:.2} MB/s", bytes_per_second as f64 / MB as f64)
    } else {
        format!("{:.2} GB/s", bytes_per_second as f64 / GB as f64)
    }
}
