use crate::errors::DownloadError;
use colored::Colorize;
use futures_util::StreamExt;
use reqwest::{header, Client};
use serde::Deserialize;
use std::fs::{File, OpenOptions};
use std::io::{Seek, SeekFrom, Write};
use std::sync::{Arc, Mutex};
use tracing::{error, info};

#[derive(Debug, Deserialize, Clone)]
pub struct DownloadRequest {
    url: String,
    filename: String,
    mime: Option<String>,
    referrer: Option<String>,
    headers: Option<Vec<String>>,
}

#[derive(Debug)]
struct DownloadInfo {
    size: u64,
    supports_parts: bool,
}

struct DownloadPart {
    url: String,
    range: Option<(u64, u64)>,
}

async fn check_download_info(
    request: &DownloadRequest,
    client: &Client,
) -> Result<DownloadInfo, DownloadError> {
    let response = match client.get(&request.url).send().await {
        Ok(response) => response,
        Err(err) => {
            error!("Failed to fetch download info: {:?}", err);
            return Err(DownloadError::DownloadInfoFetchError(
                "failed while getting response for initial request".to_string(),
            ));
        }
    };

    let size = response
        .headers()
        .get(header::CONTENT_LENGTH)
        .and_then(|val| val.to_str().ok())
        .and_then(|val| val.parse::<u64>().ok())
        .unwrap_or(0);

    let supports_parts = response.headers().get(header::ACCEPT_RANGES).is_some();

    Ok(DownloadInfo {
        size,
        supports_parts,
    })
}

fn write_to_file(
    file: Arc<Mutex<File>>,
    buf: &[u8],
    seek_position: Option<u64>,
) -> Result<(), DownloadError> {
    let mut file = file.lock().unwrap();
    if let Some(offset) = seek_position {
        file.seek(SeekFrom::Start(offset)).map_err(|e| {
            DownloadError::FileError(format!("Failed to seek to position {}: {}", offset, e))
        })?;
    }
    match file.write_all(&buf) {
        Ok(_) => Ok(()),
        Err(err) => {
            return Err(DownloadError::FileError(format!(
                "failed while writing chunk to file(buffer), Error: {}",
                err
            )));
        }
    }
}

async fn download_part(
    client: &Client,
    file: Arc<Mutex<File>>,
    part: DownloadPart,
) -> Result<(), DownloadError> {
    const BUFFER_SIZE: usize = 1024 * 1024;
    let mut chunk_sizes = Vec::<f64>::new();
    let response = match {
        match part.range {
            Some((start, end)) => {
                client
                    .get(&part.url)
                    .header(header::RANGE, format!("bytes={}-{}", start, end))
                    .send()
                    .await
            }
            None => client.get(&part.url).send().await,
        }
    } {
        Ok(response) => {
            if !response.status().is_success() {
                return Err(DownloadError::DownloadPartError(format!(
                    "failed  while downloading, result not OK, Error: {}",
                    response.status()
                )));
            } else {
                response
            }
        }
        Err(err) => {
            return Err(DownloadError::DownloadPartError(format!(
                "failed while getting response headers for downloading, Error: {}",
                err
            )));
        }
    };

    let mut stream = response.bytes_stream();

    // let mut buffer = BufWriter::new(file); // I will Create my own buffer
    let mut buffer = Vec::<u8>::with_capacity(BUFFER_SIZE);
    let mut position = match part.range {
        Some((start, _)) => start,
        None => 0,
    };

    while let Some(chunk) = stream.next().await {
        match chunk {
            Ok(chunk) => {
                chunk_sizes.push((chunk.len() as f64) / 1024.0);

                // buffer logic
                // assuming the chunk is generally less than buffer size
                if chunk.len() >= BUFFER_SIZE {
                    let mut combined = buffer.clone();
                    combined.extend_from_slice(&chunk);
                    write_to_file(file.clone(), &combined, Some(position))?;
                    position += combined.len() as u64;
                    buffer.clear();
                } else if buffer.len() + chunk.len() >= BUFFER_SIZE {
                    write_to_file(file.clone(), &buffer, Some(position))?;
                    position += buffer.len() as u64;
                    buffer.clear();
                    buffer.extend_from_slice(&chunk);
                } else {
                    buffer.extend_from_slice(&chunk);
                }
            }
            Err(err) => {
                return Err(DownloadError::DownloadPartError(format!(
                    "failed while reading chunk from stream, Error: {}",
                    err
                )));
            }
        }
    }

    // buffer logic
    write_to_file(file.clone(), &buffer, Some(position))?;
    // position += buffer.len() as u64;
    buffer.clear();

    let total_size: f64 = chunk_sizes.iter().sum();
    info!(
        "Downloaded part size: {}",
        format!("{} KB", total_size).to_string().green()
    );
    info!("Chunks: {:?}", chunk_sizes);

    Ok(())
}

pub async fn handle_download(request: DownloadRequest) -> Result<(), DownloadError> {
    info!("Download request received for {:?}", request.filename);
    info!("Checking downlaod details");

    let client = Client::new();
    let download_info = check_download_info(&request, &client).await?;

    info!(
        "Download size: {}",
        format!(
            "{} MB",
            (download_info.size as f64) / ((1024 * 1024) as f64)
        )
        .to_string()
        .green()
    );

    let file = Arc::new(Mutex::new(
        match OpenOptions::new()
            .create(true)
            .write(true)
            .open(&request.filename)
        {
            Ok(file) => file,
            Err(err) => {
                return Err(DownloadError::FileError(format!(
                    "failed while opening file for writing, Error: {}",
                    err
                )));
            }
        },
    ));

    match file.lock().unwrap().set_len(download_info.size) {
        Ok(_) => (),
        Err(err) => {
            return Err(DownloadError::FileError(format!(
                "failed while setting file length, Error: {}",
                err
            )));
        }
    }

    if download_info.supports_parts {
        info!("Download supports parts");
        let thread_count = 5;

        let part_size = download_info.size / thread_count as u64;

        let mut tasks = Vec::new();

        for i in 0..thread_count {
            let start = i * part_size;
            let end = if i == thread_count - 1 {
                download_info.size - 1
            } else {
                start + part_size - 1
            };

            let part = DownloadPart {
                url: request.url.clone(),
                range: Some((start, end)),
            };

            let file = Arc::clone(&file);
            let client = client.clone();
            let handle = tokio::spawn(async move {
                match download_part(&client, file, part).await {
                    Ok(_) => (),
                    Err(err) => {
                        error!("Failed to download part {}: {:?}", i, err);
                    }
                }
            });

            tasks.push(handle);
        }
        // let file_clone = Arc::clone(&file);
        // let handle = tokio::spawn(async move {
        //     match download_part(
        //         &client,
        //         file_clone,
        //         DownloadPart {
        //             url: request.url,
        //             range: Some((0, download_info.size - 1)),
        //         },
        //     )
        //     .await
        //     {
        //         Ok(_) => (),
        //         Err(err) => {
        //             error!("Failed to download part: {:?}", err);
        //         }
        //     }
        // });

        for handle in tasks {
            match handle.await {
                Ok(_) => (),
                Err(err) => {
                    error!("Failed to download part: {:?}", err);
                }
            }
        }
    } else {
        info!("Download does not support parts");
    }

    Ok(())
}
