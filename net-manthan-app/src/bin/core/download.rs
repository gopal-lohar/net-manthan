use crate::errors::DownloadError;
use colored::Colorize;
use futures_util::StreamExt;
use reqwest::{header, Client};
use serde::Deserialize;
use std::fs::{File, OpenOptions};
use std::io::{Seek, SeekFrom, Write};
use std::sync::{Arc, Mutex};
use std::time::Duration;
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

struct ThreadInfo {
    downloaded: u64,
    speed: u64,
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
    thread_info: Arc<Vec<Mutex<ThreadInfo>>>,
    thread_id: usize,
) -> Result<(), DownloadError> {
    const BUFFER_SIZE: usize = 1024 * 1024 * 10;
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
    let mut last_position = position;
    let mut download_speed;
    let mut last_position_time = std::time::Instant::now();

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
                if last_position != position {
                    // speed logic
                    let elapsed = last_position_time.elapsed().as_secs_f64();
                    last_position_time = std::time::Instant::now();
                    download_speed = (((position - last_position) as f64) / elapsed) as u64;
                    last_position = position;
                    thread_info[thread_id].lock().unwrap().downloaded = position;
                    thread_info[thread_id].lock().unwrap().speed = download_speed;
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
        {
            if total_size > 1024.0 {
                format!("{:.2} MB", total_size / 1024.0)
            } else {
                format!("{:.2} KB", total_size)
            }
        }
        .green()
    );
    info!("Chunks: {}", format!("{:?}", chunk_sizes).dimmed());

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
        let thread_count: usize = 5;

        let part_size = download_info.size / thread_count as u64;

        let mut tasks = Vec::new();

        let thread_info = Arc::new(
            (0..thread_count)
                .map(|_| {
                    Mutex::new(ThreadInfo {
                        downloaded: 0,
                        speed: 0,
                    })
                })
                .collect::<Vec<_>>(),
        );

        for i in 0..thread_count {
            let start = (i as u64) * part_size;
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
            let thread_info = Arc::clone(&thread_info);

            let handle = tokio::spawn(async move {
                match download_part(&client, file, part, thread_info, i).await {
                    Ok(_) => (),
                    Err(err) => {
                        error!("Failed to download part {}: {:?}", i, err);
                    }
                }
            });

            tasks.push(handle);
        }

        println!("{}", "\n".repeat(thread_count));

        let start_time = std::time::Instant::now();
        while thread_info
            .iter()
            .map(|thread| {
                let thread = thread.lock().unwrap();
                thread.downloaded
            })
            .sum::<u64>()
            < download_info.size
        {
            let stdout = std::io::stdout();
            let mut handle = stdout.lock();
            write!(handle, "{}", "\x1B[A".repeat(thread_count + 1)).unwrap();

            for (i, thread) in thread_info.iter().enumerate() {
                let thread = thread.lock().unwrap();
                write!(
                    handle,
                    "\r\tDownloaded: {} MB, Speed: {} KB/s\n",
                    format!(
                        "{:.2}",
                        ((thread.downloaded
                            - ((i as u64) * (download_info.size / (thread_count as u64))))
                            as f64)
                            / (1024.0 * 1024.0)
                    )
                    .green(),
                    format!("{:.2}", (thread.speed as f64) / 1024.0).yellow()
                )
                .unwrap();
            }
            write!(
                handle,
                "time elapsed {:?} \n",
                start_time.elapsed().as_secs()
            )
            .unwrap();

            handle.flush().unwrap();
            std::thread::sleep(Duration::from_millis(200));
        }

        for handle in tasks {
            match handle.await {
                Ok(_) => (),
                Err(err) => {
                    error!("Failed to download part: {:?}", err);
                }
            }
        }

        for d_info in thread_info.iter() {
            info!(
                "Downloaded: {} MB with speed: {} KB/s",
                format!(
                    "{:.2}",
                    (d_info.lock().unwrap().downloaded as f64) / (1024.0 * 1024.0)
                )
                .green(),
                format!("{:.2}", (d_info.lock().unwrap().speed as f64) / 1024.0)
            );
        }
    } else {
        info!("Download does not support parts");
    }

    Ok(())
}
