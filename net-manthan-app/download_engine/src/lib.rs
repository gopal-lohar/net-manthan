use crate::types::DownloadRequest;
use crate::types::PartProgress;
use chrono::DateTime;
use chrono::Duration;
use chrono::Utc;
use config::NetManthanConfig;
use crossbeam_channel::Sender;
use download_part::download_part;
use errors::DownloadError;
use get_download_info::get_download_info;
use get_download_info::DownloadInfo;
use progress_aggregator::progress_aggregator;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::path::PathBuf;
use std::sync::{atomic::AtomicBool, Arc};
use tokio::sync::Mutex;
use types::DownloadStatus;
use uuid::Uuid;

pub mod config;
pub mod download_part;
pub mod errors;
pub mod get_download_info;
pub mod progress_aggregator;
pub mod types;

/// Represents a download in the database
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Download {
    pub download_id: String,
    pub filename: String,
    pub path: String,
    pub referrer: Option<String>,
    pub download_link: String,
    pub resumable: bool,
    pub total_size: u64,
    pub size_downloaded: u64,
    pub average_speed: u64,
    pub date_added: DateTime<Utc>,
    pub active_time: u64, // Stored as seconds
    pub status: DownloadStatus,
    // Obselete from here
    pub date_finished: Option<DateTime<Utc>>,
    pub paused: bool, // New field: indicates if the download is currently paused
    pub error: bool,  // New field: indicates if the download has encountered an error
    // Obselete till here
    pub parts: Vec<DownloadPart>,
}

/// Represents a part of a download in the database
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DownloadPart {
    pub download_id: String,
    pub part_id: String,
    pub start_bytes: u64,
    pub end_bytes: u64,
    pub total_bytes: u64,
    pub bytes_downloaded: u64,
}

fn create_download_file(filepath: &PathBuf, size: u64) -> Result<(), DownloadError> {
    let mut path = filepath.clone();
    if path.exists() {
        let mut count = 1;
        let parent = path.parent().unwrap_or_else(|| std::path::Path::new(""));
        let stem = path.file_stem().unwrap_or_default().to_string_lossy();
        let ext = path.extension().unwrap_or_default().to_string_lossy();

        loop {
            let new_name = format!("{} ({})", stem, count);
            let new_path = if ext.is_empty() {
                parent.join(new_name)
            } else {
                parent.join(format!("{}.{}", new_name, ext))
            };

            if !new_path.exists() {
                path = new_path;
                break;
            }
            count += 1;
        }
    }

    let file = match File::create(path) {
        Ok(file) => file,
        Err(err) => return Err(DownloadError::FileSystemError(err)),
    };

    match file.set_len(size) {
        Ok(_) => Ok(()),
        Err(err) => Err(DownloadError::FileSystemError(err)),
    }
}

fn get_download_parts(
    thread_count: u8,
    download_id: &String,
    download_info: &DownloadInfo,
) -> Vec<DownloadPart> {
    let mut parts = Vec::<DownloadPart>::new();
    if thread_count == 1 || download_info.resume == false {
        parts.push(DownloadPart {
            download_id: download_id.clone(),
            part_id: Uuid::new_v4().to_string(),
            start_bytes: 0,
            end_bytes: download_info.size - 1,
            total_bytes: download_info.size,
            bytes_downloaded: 0,
        });
        return parts;
    }

    let part_size = download_info.size as f64 / thread_count as f64;
    for i in 0..thread_count {
        let part = DownloadPart {
            download_id: download_id.clone(),
            part_id: Uuid::new_v4().to_string(),
            start_bytes: (part_size * i as f64).round() as u64,
            end_bytes: (part_size * (i + 1) as f64).round() as u64,
            total_bytes: part_size.round() as u64,
            bytes_downloaded: 0,
        };
        parts.push(part);
    }
    parts
}

impl Download {
    pub async fn new(
        request: DownloadRequest,
        config: NetManthanConfig,
    ) -> Result<Self, DownloadError> {
        let download_id = uuid::Uuid::new_v4().to_string();
        let download_info = get_download_info(&request).await?;

        // create file
        let mut filepath = request
            .filepath
            .unwrap_or_else(|| config.download_dir.clone());
        let filename_from_info = download_info.filename.clone();
        let filename = request.filename.unwrap_or_else(|| {
            filename_from_info.unwrap_or_else(|| format!("unknow_download_{}", download_id))
        });
        filepath = filepath.join(filename.clone());
        create_download_file(&filepath, download_info.size)?;

        let parts = get_download_parts(config.thread_count, &download_id, &download_info);

        let fallback_download_dir = config.download_dir.to_str().unwrap_or_else(|| "");
        let fallback_file_path =
            format!("{}unknow_download_{}", fallback_download_dir, download_id);

        Ok(Download {
            download_id,
            filename: filename.clone(),
            // filpath will not be None most probably
            path: filepath
                .to_str()
                .unwrap_or_else(|| &fallback_file_path)
                .to_string(),
            referrer: request.referrer,
            download_link: request.url,
            resumable: download_info.resume,
            total_size: download_info.size,
            size_downloaded: 0,
            average_speed: 0,
            date_added: Utc::now(),
            status: DownloadStatus::Connecting,
            date_finished: None,
            active_time: 0,
            paused: true,
            error: false,
            parts,
        })
    }

    pub async fn start(
        &mut self,
        aggregator_sender: Sender<Vec<PartProgress>>,
        config: NetManthanConfig,
        cancel_token: Arc<AtomicBool>,
    ) {
        let update_interval = Duration::milliseconds(config.update_interval_in_ms);
        let buffer_size = (if self.parts.len() > 1 {
            config.multi_threaded_buffer_size_in_kb
        } else {
            config.single_threaded_buffer_size_in_kb
        }) * 1024;

        let mut part_progress_vec = Vec::with_capacity(self.parts.len());

        for part in &self.parts {
            let cancel_token = cancel_token.clone();
            let range = if self.resumable && self.parts.len() > 1 {
                Some((part.start_bytes, part.end_bytes))
            } else if self.resumable == false && self.parts.len() == 1 {
                None
            } else {
                None
            };

            let progress = Arc::new(Mutex::new(PartProgress {
                part_id: part.part_id.clone(),
                bytes_downloaded: part.bytes_downloaded,
                speed_in_bytes: 0,
                status: DownloadStatus::Connecting,
            }));

            part_progress_vec.push(progress.clone());

            tokio::spawn(download_part(
                self.download_link.clone(),
                None, // TODO: Implement headers, both while checking the download and while downloading
                range,
                part.bytes_downloaded,
                PathBuf::from(self.path.clone()),
                buffer_size,
                progress,
                Duration::milliseconds(config.update_interval_in_ms),
                cancel_token,
            ));
        }

        self.status = DownloadStatus::Downloading;

        tokio::spawn(progress_aggregator(
            part_progress_vec,
            aggregator_sender,
            update_interval,
            cancel_token,
        ));
    }
}
