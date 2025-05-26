use crate::download_config::DownloadConfig;
use crate::errors::DownloadError;
use crate::types::DownloadRequest;
use crate::utils::{calculate_chunks, extract_filename};
use crate::{
    ByteRange, DownloadPart, NonResumableDownloadPart, PartProgress, ResumableDownloadPart,
    TotalSize,
};
use crate::{download_part::DownloadParts, types::DownloadStatus, utils::format_bytes};
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
    /// Current status of the download
    status: DownloadStatus,
    /// Arc Bool token for pausing the download.
    pub stop_token: Arc<AtomicBool>,
    /// configuration options for the download.
    pub config: DownloadConfig,
    /// Parts of the download.
    pub parts: DownloadParts,
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
            parts: DownloadParts::Created,
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
                        part: DownloadPart {
                            id: Uuid::new_v4(),
                            status: DownloadStatus::Queued,
                            bytes_downloaded: 0,
                            current_speed: 0,
                            progress: Arc::new(Mutex::new(PartProgress {
                                status: DownloadStatus::Queued,
                                bytes_downloaded: 0,
                                current_speed: 0,
                            })),
                        },
                        range: ByteRange {
                            start_byte: *start_byte,
                            end_byte: *end_byte,
                        },
                    })
                    .collect(),
            )
        } else {
            DownloadParts::NonResumable(NonResumableDownloadPart {
                part: DownloadPart {
                    id: Uuid::new_v4(),
                    status: DownloadStatus::Queued,
                    bytes_downloaded: 0,
                    current_speed: 0,
                    progress: Arc::new(Mutex::new(PartProgress {
                        status: DownloadStatus::Queued,
                        bytes_downloaded: 0,
                        current_speed: 0,
                    })),
                },
                total_size,
            })
        };

        self.status = DownloadStatus::Queued;

        if total_size != self.get_total_size() {
            self.parts = DownloadParts::ErrorLoadingInfo;
            return Err(DownloadError::GeneralError("Mismatch in total size".into()));
        }

        Ok(())
    }
}
