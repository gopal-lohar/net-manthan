use crate::types::DownloadRequest;
use crate::types::PartProgress;
use crossbeam_channel::bounded;
use crossbeam_channel::Sender;
use download_part::download_part;
use errors::DownloadError;
use get_download_info::get_download_info;
use get_download_info::DownloadInfo;
use progress_aggregator::progress_aggregator;
use std::path::PathBuf;
use std::{
    fs::OpenOptions,
    sync::{atomic::AtomicBool, Arc},
};
use types::DownloadPart;

pub mod config;
pub mod download_part;
pub mod errors;
pub mod get_download_info;
pub mod progress_aggregator;
pub mod types;

fn create_download_file(filepath: &PathBuf, size: u64) -> Result<(), DownloadError> {
    match OpenOptions::new().create(true).write(true).open(filepath) {
        Ok(handle) => match handle.set_len(size) {
            Ok(_) => Ok(()),
            Err(err) => Err(DownloadError::FileSystemError(err)),
        },
        Err(err) => Err(DownloadError::FileSystemError(err)),
    }
}

fn get_download_parts(thread_count: u8, download_info: DownloadInfo) -> Vec<DownloadPart> {
    let mut parts = Vec::<DownloadPart>::new();
    for i in 0..thread_count {
        let part = DownloadPart {
            part_id: i,
            bytes_downloaded: 0,
            range: match download_info.resume {
                true => Some((
                    (download_info.size as f64 / thread_count as f64 * i as f64).round() as u64,
                    (download_info.size as f64 / thread_count as f64 * (i + 1) as f64).round()
                        as u64,
                )),
                false => None,
            },
        };
        parts.push(part);
    }
    parts
}

pub async fn download(
    mut request: DownloadRequest,
    cancel_token: Arc<AtomicBool>,
    progress_sender: Sender<Vec<PartProgress>>,
) -> Result<(), DownloadError> {
    let mut download_handles = Vec::new();
    let (aggregator_sender, aggregator_receiver) = bounded::<PartProgress>(100);
    let download_info = get_download_info(&request).await?;
    create_download_file(&request.filepath, download_info.size)?;

    let parts = match request.parts {
        Some(parts) => parts,
        None => get_download_parts(request.config.thread_count, download_info),
    };

    {
        let initial_progress = parts
            .iter()
            .map(|part| PartProgress {
                part_id: part.part_id,
                bytes_downloaded: part.bytes_downloaded,
                total_bytes: part.range.map(|(start, end)| end - start + 1).unwrap_or(0),
                speed: 0,
                timestamp: chrono::Utc::now(),
                error: false,
            })
            .collect::<Vec<PartProgress>>();
        let cancel_token = cancel_token.clone();
        let progress_receiver = aggregator_receiver.clone();
        tokio::spawn(progress_aggregator(
            initial_progress,
            progress_receiver,
            progress_sender,
            request.config.update_interval,
            cancel_token,
        ));
    }

    // deliberately set parts to None, so that it can't be used later
    request.parts = None;

    for part in parts {
        let aggregator_sender = aggregator_sender.clone();
        let cancel_token = cancel_token.clone();
        let handle = tokio::spawn(download_part(
            request.clone(),
            part,
            aggregator_sender,
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

    cancel_token.store(true, std::sync::atomic::Ordering::Relaxed);
    Ok(())
}
