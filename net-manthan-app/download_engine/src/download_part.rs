use crate::{
    errors::DownloadError,
    types::{DownloadStatus, PartProgress},
};
use chrono::{Duration, Utc};
use futures_util::StreamExt;
use reqwest::{
    header::{self, HeaderMap, HeaderName, HeaderValue},
    Client,
};
use std::{
    fs::{File, OpenOptions},
    io::{BufWriter, Seek, Write},
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};
use tokio::sync::Mutex;

fn open_download_file(
    filepath: PathBuf,
    range: Option<(u64, u64)>,
    bytes_downloaded: u64,
    buffer_size: u64,
) -> Result<BufWriter<File>, std::io::Error> {
    let mut file = OpenOptions::new().write(true).open(filepath)?;
    match range {
        Some((start, _)) => {
            file.seek(std::io::SeekFrom::Start(start + bytes_downloaded))?;
        }
        None => {}
    }
    let file_writer = BufWriter::with_capacity(buffer_size as usize, file);
    Ok(file_writer)
}

// TODO: it is ran in a tokio::task so doesn't mean anything by returning error for now (SO DON'T)
pub async fn download_part(
    url: String,
    headers: Option<Vec<String>>,
    range: Option<(u64, u64)>,
    mut bytes_downloaded: u64,
    filepath: PathBuf,
    buffer_size: u64,
    progress: Arc<Mutex<PartProgress>>,
    update_interval: Duration,
    cancel_token: Arc<AtomicBool>,
) -> Result<(), DownloadError> {
    let mut file_writer = match open_download_file(filepath, range, bytes_downloaded, buffer_size) {
        Ok(file) => file,
        Err(err) => {
            return Err(DownloadError::FileSystemError(err));
        }
    };

    let client = Client::new();

    let mut req = client.get(&url);

    if let Some(headers) = headers {
        let mut header_map = HeaderMap::new();
        for header in headers {
            if let Some((name, value)) = header.split_once(": ") {
                if let (Ok(name), Ok(value)) = (
                    HeaderName::from_bytes(name.as_bytes()),
                    HeaderValue::from_str(value),
                ) {
                    header_map.insert(name, value);
                }
            }
        }
        req = req.headers(header_map);
    }

    let sent_request = match range {
        Some((start, end)) => {
            req.header(header::RANGE, format!("bytes={}-{}", start, end))
                .send()
                .await
        }
        None => req.send().await,
    };

    let response = match sent_request {
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
                if elapsed >= update_interval {
                    speed_in_bytes =
                        ((bytes_downloaded_last as f64) / elapsed.num_seconds() as f64) as u64;
                    // TODO: sync this with the bufwriter, somehow
                    bytes_downloaded += bytes_downloaded_last;
                    bytes_downloaded_last = 0;
                    last_update_time = Utc::now();

                    // update progress
                    let mut progress_unlocked = progress.lock().await;
                    progress_unlocked.bytes_downloaded = bytes_downloaded;
                    progress_unlocked.speed_in_bytes = speed_in_bytes;
                    progress_unlocked.status = DownloadStatus::Downloading;

                    if cancel_token.load(Ordering::Relaxed) {
                        // TODO: handle cancellation properly
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

    // update progress
    let mut progress_unlocked = progress.lock().await;
    progress_unlocked.bytes_downloaded = bytes_downloaded;
    progress_unlocked.speed_in_bytes = speed_in_bytes;
    progress_unlocked.status = DownloadStatus::Completed(Utc::now());
    Ok(())
}
