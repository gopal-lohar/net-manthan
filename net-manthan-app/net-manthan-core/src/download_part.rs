use crate::{
    errors::DownloadError,
    types::{DownloadPart, DownloadRequest, PartProgress},
};
use chrono::Utc;
use crossbeam_channel::Sender;
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

fn open_download_file(
    filepath: PathBuf,
    range: Option<(u64, u64)>,
    bytes_downloaded: u64,
    buffer_size: usize,
) -> Result<BufWriter<File>, std::io::Error> {
    let mut file = OpenOptions::new().write(true).open(filepath)?;
    match range {
        Some((start, _)) => {
            file.seek(std::io::SeekFrom::Start(start + bytes_downloaded))?;
        }
        None => {}
    }
    let file_writer = BufWriter::with_capacity(buffer_size, file);
    Ok(file_writer)
}

pub async fn download_part(
    request: DownloadRequest,
    part: DownloadPart,
    progress_sender: Sender<PartProgress>,
    cancel_token: Arc<AtomicBool>,
) -> Result<(), DownloadError> {
    let mut bytes_downloaded: u64 = part.bytes_downloaded;

    let mut file_writer = match open_download_file(
        request.filepath,
        part.range,
        bytes_downloaded,
        request.config.buffer_size,
    ) {
        Ok(file) => file,
        Err(err) => {
            return Err(DownloadError::FileSystemError(err));
        }
    };

    let client = Client::new();

    let mut req = client.get(&request.url);

    if let Some(headers) = &request.headers {
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

    let sent_request = match part.range {
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
                if elapsed >= request.config.update_interval {
                    speed_in_bytes =
                        ((bytes_downloaded_last as f64) / elapsed.num_seconds() as f64) as u64;
                    bytes_downloaded += bytes_downloaded_last;
                    bytes_downloaded_last = 0;
                    last_update_time = Utc::now();

                    let progress = PartProgress {
                        part_id: part.part_id,
                        bytes_downloaded,
                        total_bytes: download_size,
                        speed: speed_in_bytes,
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
    let progress = PartProgress {
        part_id: part.part_id,
        bytes_downloaded,
        total_bytes: download_size,
        speed: speed_in_bytes,
        timestamp: Utc::now(),
        error: false,
    };
    match progress_sender.send(progress) {
        Ok(_) => {}
        Err(_) => {}
    }
    Ok(())
}
