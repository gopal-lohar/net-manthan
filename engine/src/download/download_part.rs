use crate::types::{
    chunks::{DownloadPart, SizeInfo},
    errors::DownloadError,
    others::DownloadConfig,
    request::{DownloadInfo, DownloadRequest},
    status::DownloadStatus,
};
use chrono::{DateTime, Utc};
use futures_util::StreamExt;
use reqwest::Client;
use std::{io::SeekFrom, path::PathBuf, sync::Arc};
use tokio::{
    fs::OpenOptions,
    io::AsyncSeekExt,
    sync::{Mutex, watch},
};
use tracing::warn;

use super::custom_writer::CustomWriter;

// we don't want to return anything from this function
// we don't want to mutate anything other than the part_progress which is mutex
/// downloads a part and update it's progress (does not do anything if info is not loaded)
pub async fn download_part<T: SizeInfo>(
    request: DownloadRequest,
    info: DownloadInfo,
    part: DownloadPart<T>,
    part_progress: Arc<Mutex<DownloadPart<T>>>,
    config: DownloadConfig,
    pause_rx: watch::Receiver<bool>,
) {
    let mut update_manager = UpdateManager::new(part_progress);
    let file_path = request.get_file_path(&info);
    if let Err(err) = download_part_inner(
        request,
        part,
        file_path,
        &mut update_manager,
        config,
        pause_rx,
    )
    .await
    {
        update_manager.update_error(err).await;
    }
}

async fn download_part_inner<T: SizeInfo>(
    request: DownloadRequest,
    part: DownloadPart<T>,
    file_path: PathBuf,
    update_manager: &mut UpdateManager<T>,
    config: DownloadConfig,
    mut pause_rx: watch::Receiver<bool>,
) -> Result<(), DownloadError> {
    // open file
    let mut file = OpenOptions::new().write(true).open(file_path).await?;
    file.seek(SeekFrom::Start(part.get_write_head())).await?;
    let mut writer = CustomWriter::with_capacity(file, config.buffer_size).await;

    // connect to server
    let client = Client::new();
    let mut request = request.build_request(&client);
    request = part.apply_header(request);
    let response = request.send().await?;

    // process response
    let mut stream = response.bytes_stream();

    loop {
        tokio::select! {
            // Check for cancellation
            _ = pause_rx.changed() => {
                if *pause_rx.borrow() {
                    break;
                }
            }

            // Process next chunk
            chunk_result = stream.next() => {
                match chunk_result {
                    Some(chunk) => {
                        let chunk = chunk?;
                        let bytes_written = writer.write_all(&chunk).await?;
                        update_manager.update_progress(bytes_written).await;
                    }
                    None => break,
                }
            }
        }
    }

    let bytes_written = writer.flush().await?;
    update_manager.update_progress(bytes_written).await;
    let paused = *pause_rx.borrow();
    update_manager.update_completed(paused).await;
    Ok(())
}

struct UpdateManager<T: SizeInfo> {
    part_progress: Arc<Mutex<DownloadPart<T>>>,
    last_flush_time: DateTime<Utc>,
}

impl<T: SizeInfo> UpdateManager<T> {
    fn new(part_progress: Arc<Mutex<DownloadPart<T>>>) -> Self {
        Self {
            part_progress,
            last_flush_time: Utc::now(),
        }
    }

    async fn update_progress(&mut self, bytes_written: Option<u64>) {
        if let Some(bytes_written) = bytes_written {
            let now = Utc::now();
            let delta = (now - self.last_flush_time).num_milliseconds().abs() as u64;
            let speed = if delta == 0 {
                bytes_written
            } else {
                bytes_written / delta
            };
            self.last_flush_time = now;

            {
                let mut part_progress = self.part_progress.lock().await;
                part_progress.bytes_downloaded += bytes_written;
                part_progress.current_speed = speed;
            }
        }
    }

    async fn update_completed(&mut self, pause_rx: bool) {
        {
            let mut part_progress = self.part_progress.lock().await;
            if pause_rx {
                part_progress.status = DownloadStatus::Paused;
            }
            if let Some(total_size) = part_progress.total_size() {
                if part_progress.bytes_downloaded == total_size {
                    part_progress.status = DownloadStatus::Complete;
                }
            }
        }
    }

    async fn update_error(&mut self, error: DownloadError) {
        {
            let mut part_progress = self.part_progress.lock().await;
            warn!("Download error: {}", error);
            part_progress.error = Some(error.to_string());
            part_progress.status = DownloadStatus::Failed;
        }
    }
}
