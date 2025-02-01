use crate::{errors::DownloadError, types::ChunkProgress};
use chrono::{Duration, Utc};
use crossbeam_channel::Sender;
use futures_util::StreamExt;
use reqwest::{header, Client};
use std::{
    fs::{File, OpenOptions},
    io::{BufWriter, Seek, Write},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

const BUFFER_SIZE: usize = 1024 * 1024;

pub struct DownloadPart {
    pub download_id: u64,
    pub chunk_id: u32,
    pub url: String,
    pub filepath: String,
    pub bytes_downloaded: u64,
    pub range: Option<(u64, u64)>,
}

fn open_download_file(
    filepath: &str,
    range: Option<(u64, u64)>,
    bytes_downloaded: u64,
) -> Result<BufWriter<File>, std::io::Error> {
    let mut file = OpenOptions::new().write(true).open(filepath)?;
    match range {
        Some((start, _)) => {
            file.seek(std::io::SeekFrom::Start(start + bytes_downloaded))?;
        }
        None => {}
    }
    let file_writer = BufWriter::with_capacity(BUFFER_SIZE, file);
    Ok(file_writer)
}

pub async fn download_part(
    part: DownloadPart,
    progress_sender: Sender<ChunkProgress>,
    udpate_interval: Duration,
    cancel_token: Arc<AtomicBool>,
) -> Result<(), DownloadError> {
    let mut bytes_downloaded: u64 = part.bytes_downloaded;

    let mut file_writer = match open_download_file(&part.filepath, part.range, bytes_downloaded) {
        Ok(file) => file,
        Err(err) => {
            return Err(DownloadError::FileSystemError(err));
        }
    };

    let client = Client::new();
    let reqeust = match part.range {
        Some((start, end)) => {
            client
                .get(&part.url)
                .header(header::RANGE, format!("bytes={}-{}", start, end))
                .send()
                .await
        }
        None => client.get(&part.url).send().await,
    };

    let response = match reqeust {
        Ok(response) => {
            if !response.status().is_success() {
                return Err(DownloadError::GeneralError(format!(
                    "failed while downloading, HTTP status code: {}",
                    response.status()
                )));
            } else {
                response
            }
        }
        Err(err) => {
            return Err(DownloadError::HttpRequestError(err));
        }
    };

    let download_size = response
        .headers()
        .get(header::CONTENT_LENGTH)
        .and_then(|val| val.to_str().ok())
        .and_then(|val| val.parse::<u64>().ok())
        .unwrap_or(0);

    let mut stream = response.bytes_stream();

    let start_time = Utc::now();
    let mut last_update_time = start_time;
    let mut bytes_downloaded_last: u64 = 0;
    let mut speed_in_bytes: u64 = 0;

    while let Some(chunk) = stream.next().await {
        match chunk {
            Ok(chunk) => {
                file_writer
                    .write_all(&chunk)
                    .map_err(DownloadError::from_write_error)?;
                bytes_downloaded_last += chunk.len() as u64;
                let elapsed = Utc::now() - last_update_time;
                if elapsed >= udpate_interval {
                    speed_in_bytes =
                        ((bytes_downloaded_last as f64) / elapsed.num_seconds() as f64) as u64;
                    bytes_downloaded += bytes_downloaded_last;
                    bytes_downloaded_last = 0;
                    last_update_time = Utc::now();

                    let progress = ChunkProgress {
                        download_id: part.download_id,
                        chunk_id: part.chunk_id,
                        bytes_downloaded,
                        total_bytes: download_size,
                        speed: speed_in_bytes as f64,
                        timestamp: Utc::now(),
                        error: false,
                    };

                    if progress_sender.send(progress).is_err()
                        || cancel_token.load(Ordering::Relaxed)
                    {
                        break;
                    }
                }
            }
            Err(err) => {
                return Err(DownloadError::HttpRequestError(err));
            }
        }
    }

    file_writer
        .flush()
        .map_err(DownloadError::from_write_error)?;

    bytes_downloaded += bytes_downloaded_last;
    let progress = ChunkProgress {
        download_id: part.download_id,
        chunk_id: part.chunk_id,
        bytes_downloaded,
        total_bytes: download_size,
        speed: speed_in_bytes as f64,
        timestamp: Utc::now(),
        error: false,
    };
    match progress_sender.send(progress) {
        Ok(_) => {}
        Err(_) => {}
    }
    Ok(())
}
