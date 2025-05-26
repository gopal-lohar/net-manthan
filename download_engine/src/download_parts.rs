use crate::{
    download_part::{DownloadPart, NonResumableDownloadPart, ResumableDownloadPart},
    types::DownloadStatus,
};

// the first three states help in calculating status of download
#[derive(Clone, Debug)]
pub enum DownloadParts {
    Created,
    LoadingInfo,
    ErrorLoadingInfo,
    Resumable(Vec<ResumableDownloadPart>),
    NonResumable(NonResumableDownloadPart),
}

impl DownloadParts {
    // pub fn get_parts(&self) -> Vec<&DownloadPart> {
    //     match self {
    //         DownloadParts::Resumable(parts) => parts.iter().map(|p| &p.part).collect(),
    //         DownloadParts::NonResumable(part) => vec![&part.part],
    //         _ => vec![],
    //     }
    // }

    // pub fn get_parts_mut(&mut self) -> Vec<&mut DownloadPart> {
    //     match self {
    //         DownloadParts::Resumable(parts) => parts.iter_mut().map(|p| &mut p.part).collect(),
    //         DownloadParts::NonResumable(part) => vec![&mut part.part],
    //         _ => vec![],
    //     }
    // }

    // For immutable access
    pub fn as_download_parts(&self) -> Vec<&DownloadPart> {
        match self {
            DownloadParts::Resumable(parts) => {
                parts.iter().map(|p| DownloadPart::Resumable(p)).collect()
            }
            DownloadParts::NonResumable(part) => vec![DownloadPart::NonResumable(part)],
            _ => vec![],
        }
    }

    // For mutable access
    pub fn as_download_parts_mut(&mut self) -> Vec<&mut DownloadPart> {
        match self {
            DownloadParts::Resumable(parts) => parts
                .iter_mut()
                .map(|p| DownloadPart::Resumable(p))
                .collect(),
            DownloadParts::NonResumable(part) => vec![DownloadPart::NonResumable(part)],
            _ => vec![],
        }
    }

    pub fn get_total_size(&self) -> u64 {
        match self {
            DownloadParts::Resumable(parts) => parts.iter().map(|p| p.get_total_size()).sum(),
            DownloadParts::NonResumable(part) => part.get_total_size(),
            _ => 0,
        }
    }

    pub fn is_resumable(&self) -> bool {
        matches!(self, DownloadParts::Resumable(_))
    }

    pub fn total_bytes_downloaded(&self) -> u64 {
        match self {
            DownloadParts::Resumable(parts) => {
                parts.iter().map(|p| p.common.bytes_downloaded).sum()
            }
            DownloadParts::NonResumable(part) => part.common.bytes_downloaded,
            _ => 0,
        }
    }

    pub fn set_status(&mut self, status: DownloadStatus) {
        match self {
            DownloadParts::Resumable(parts) => {
                for part in parts {
                    part.common.status = status.clone();
                }
            }
            DownloadParts::NonResumable(part) => part.common.status = status,
            _ => {}
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

    pub fn get_bytes_downloaded(&self) -> u64 {
        match &self.parts {
            DownloadParts::Resumable(parts) => parts.iter().map(|part| part.bytes_downloaded).sum(),
            DownloadParts::NonResumable(part) => part.bytes_downloaded,
            DownloadParts::None(_) => 0,
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

    pub async fn update_progress_old(&mut self) {
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
