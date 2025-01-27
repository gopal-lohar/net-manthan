use std::{
    fs::OpenOptions,
    sync::{atomic::AtomicBool, Arc},
    time::Duration,
};

use crossbeam_channel::bounded;
use download_part::{download_part, ChunkProgress};
use errors::DownloadError;
use get_download_info::get_download_info;
use progress_aggregator::progress_aggregator;
use tracing::info;
use utils::DownloadRequest;

pub mod download_part;
pub mod errors;
pub mod get_download_info;
pub mod progress_aggregator;

fn create_download_file(filename: &String, size: u64) -> Result<(), DownloadError> {
    match OpenOptions::new().create(true).write(true).open(filename) {
        Ok(handle) => match handle.set_len(size) {
            Ok(_) => Ok(()),
            Err(err) => Err(DownloadError::FileSystemError(err)),
        },
        Err(err) => Err(DownloadError::FileSystemError(err)),
    }
}

pub async fn download(request: DownloadRequest) -> Result<(), DownloadError> {
    const DOWNLOAD_ID: u64 = 69;
    const THREAD_COUNT: usize = 5;

    let cancel_token = Arc::new(AtomicBool::new(false));
    let mut download_handles = Vec::new();
    let (aggregator_sender, aggregator_receiver) = bounded::<ChunkProgress>(100);
    let (download_chunks, download_size) =
        get_download_info(DOWNLOAD_ID, THREAD_COUNT, &request).await?;

    info!("Download starting");
    create_download_file(&request.filename, download_size)?;

    {
        let cancel_token = cancel_token.clone();
        let progress_receiver = aggregator_receiver.clone();
        tokio::spawn(progress_aggregator(
            DOWNLOAD_ID,
            progress_receiver,
            Duration::from_millis(500),
            cancel_token,
        ));
    }

    for chunk in download_chunks {
        let aggregator_sender = aggregator_sender.clone();
        let cancel_token = cancel_token.clone();
        let handle = tokio::spawn(download_part(
            chunk,
            aggregator_sender,
            Duration::from_millis(1000),
            cancel_token,
        ));
        download_handles.push(handle);
    }

    for handle in download_handles {
        match handle.await {
            Ok(part) => {
                if let Err(err) = part {
                    cancel_token.store(true, std::sync::atomic::Ordering::Relaxed);
                    return Err(err);
                }
            }
            Err(_) => {
                cancel_token.store(true, std::sync::atomic::Ordering::Relaxed);
                return Err(DownloadError::DownloadInterrupted);
            }
        }
    }

    Ok(())
}
