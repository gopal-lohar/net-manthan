pub mod errors;
use colored::Colorize;
use errors::DownloadError;
use futures_util::StreamExt;
use reqwest::{header, Client};
use serde::Deserialize;
use std::{
    fs::OpenOptions,
    io::{BufWriter, Seek, Write},
    time::Duration,
};
use tokio::sync::mpsc;
use tracing::{error, info, warn};

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
    resume: bool,
}

struct DownloadPart {
    url: String,
    range: Option<(u64, u64)>,
    filename: String,
}

#[derive(Clone)]
struct DownloadFinished {
    completed: bool,
    error: bool,
    average_chunk_size: f64,
}

#[derive(Clone)]
struct DownloadProgress {
    progress: u64,
    download_size: u64,
    speed: u64,
    time_elapsed: Duration,
    thread_id: usize,
    finished: Option<DownloadFinished>,
}

async fn check_download_info(request: &DownloadRequest) -> Result<DownloadInfo, DownloadError> {
    let client = Client::new();
    let response = match client.get(&request.url).send().await {
        Ok(response) => response,
        Err(err) => {
            return Err(DownloadError::HttpRequestError(err));
        }
    };

    let size = response
        .headers()
        .get(header::CONTENT_LENGTH)
        .and_then(|val| val.to_str().ok())
        .and_then(|val| val.parse::<u64>().ok())
        .unwrap_or(0);

    let resume = response.headers().get(header::ACCEPT_RANGES).is_some();

    Ok(DownloadInfo { size, resume })
}

fn format_bytes(size_in_bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if size_in_bytes < MB {
        let kb = size_in_bytes as f64 / KB as f64;
        if kb.fract() == 0.0 {
            format!("{:.0} KB", kb) // No decimals if fractional part is 0
        } else {
            format!("{:.2} KB", kb)
        }
    } else if size_in_bytes < GB {
        let mb = size_in_bytes as f64 / MB as f64;
        if mb.fract() == 0.0 {
            format!("{:.0} MB", mb)
        } else {
            format!("{:.2} MB", mb)
        }
    } else {
        let gb = size_in_bytes as f64 / GB as f64;
        if gb.fract() == 0.0 {
            format!("{:.0} GB", gb)
        } else {
            format!("{:.2} GB", gb)
        }
    }
}

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
    const THREAD_COUNT: usize = 5;

    let _ = request.mime;
    let _ = request.referrer;
    let _ = request.headers;

    info!("Checking downlaod details");
    let download_info = check_download_info(&request).await?;
    info!(
        "Download details: {}",
        format_bytes(download_info.size).green()
    );

    info!("Creating download file");
    create_download_file(&request.filename, download_info.size)?;
    info!("Download file created");

    let thread_count = match download_info.resume {
        true => {
            info!("Resumable download supported");
            THREAD_COUNT
        }
        false => {
            warn!("Resumable download not supported");
            1
        }
    };

    let (tx, rx) = mpsc::channel::<DownloadProgress>(10);
    for i in 0..thread_count {
        let tx = tx.clone();
        let part = DownloadPart {
            url: request.url.clone(),
            range: match download_info.resume {
                true => Some((
                    (download_info.size as f64 / thread_count as f64 * i as f64).round() as u64,
                    (download_info.size as f64 / thread_count as f64 * (i + 1) as f64).round()
                        as u64,
                )),
                false => None,
            },
            filename: request.filename.clone(),
        };
        tokio::spawn(download_part(part, tx, Duration::from_millis(500), i));
    }

    log_prgress(rx, thread_count).await;

    Ok(())
}

async fn log_prgress(mut rx: mpsc::Receiver<DownloadProgress>, thread_count: usize) {
    println!("");

    let stdout = std::io::stdout();
    let mut handle = stdout.lock();

    let hash_count = 40;
    let mut write_thread_info = |download: DownloadProgress| {
        write!(
            handle,
            "{}\r{}: [{}{}] {} / {} Speed: {}/s {}{}{}\n{}",
            "\x1B[B".repeat(download.thread_id),
            download.thread_id,
            "#".repeat(
                (download.progress as f64 / download.download_size as f64 * hash_count as f64)
                    .round() as usize
            )
            .green(),
            " ".repeat(
                hash_count
                    - (download.progress as f64 / download.download_size as f64 * hash_count as f64)
                        .round() as usize
            ),
            format_bytes(download.progress).green(),
            format_bytes(download.download_size),
            format_bytes(download.speed),
            {
                let time_elapsed = download.time_elapsed.as_secs();
                let minutes = (time_elapsed % 3600) / 60;
                let seconds = time_elapsed % 60;
                format!("{:02}:{:02}", minutes, seconds)
            },
            {
                match download.finished {
                    Some(finished) => {
                        if finished.completed {
                            format!("[{}]", "Completed".green())
                        } else {
                            format!("[{}]", "Error".red())
                        }
                    }
                    None => "".to_string(),
                }
            },
            " ".repeat(10),
            "\x1B[A".repeat(download.thread_id + 1)
        )
        .unwrap();
    };

    let mut download_progress = vec![
        DownloadProgress {
            progress: 0,
            download_size: 0,
            speed: 0,
            time_elapsed: Duration::from_secs(0),
            finished: None,
            thread_id: 0
        };
        thread_count
    ];

    for i in 0..thread_count {
        write_thread_info(download_progress[i].clone());
    }

    while let Some(download) = rx.recv().await {
        download_progress[download.thread_id] = download.clone();
        write_thread_info(download.clone());
        if download_progress.iter().all(|part| part.finished.is_some()) {
            break;
        }
    }

    println!(
        "Average chunk size for threads {}",
        format_bytes(
            (download_progress
                .iter()
                .map(|part| part.finished.as_ref().unwrap().average_chunk_size)
                .sum::<f64>()
                / thread_count as f64) as u64
        )
        .green()
    );

    if download_progress
        .iter()
        .any(|part| part.finished.as_ref().unwrap().error)
    {
        println!("{}", "Download failed".red());
    }

    println!("{}", "\x1B[B".repeat(5));
}

async fn download_part(
    part: DownloadPart,
    tx: mpsc::Sender<DownloadProgress>,
    update_interval: Duration,
    thread_id: usize,
) -> Result<(), DownloadError> {
    const BUFFER_SIZE: usize = 1024 * 1024;
    let mut chunk_sizes = Vec::<f64>::new();
    let mut last_update_time = std::time::Instant::now();
    let mut bytes_transferred_since_update: u64 = 0;
    let mut progress: u64 = 0;
    let mut speed: u64 = 0;

    let client = Client::new();

    let file = match OpenOptions::new().write(true).open(part.filename) {
        Ok(mut file) => match part.range {
            Some((start, _)) => match file.seek(std::io::SeekFrom::Start(start)) {
                Ok(_) => file,
                Err(err) => {
                    return Err(DownloadError::FileSystemError(err));
                }
            },
            None => file,
        },
        Err(err) => {
            return Err(DownloadError::FileSystemError(err));
        }
    };
    let mut file_writer = BufWriter::with_capacity(BUFFER_SIZE, file);

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

    match tx
        .send(DownloadProgress {
            progress,
            download_size,
            speed,
            time_elapsed: Duration::from_secs(0),
            finished: None,
            thread_id,
        })
        .await
    {
        Ok(_) => {}
        Err(err) => {
            error!("Error sending progress: {}", err);
        }
    }

    let mut stream = response.bytes_stream();

    let start_time = std::time::Instant::now();

    while let Some(chunk) = stream.next().await {
        match chunk {
            Ok(chunk) => {
                chunk_sizes.push((chunk.len() as f64) / 1024.0);
                file_writer
                    .write_all(&chunk)
                    .map_err(DownloadError::from_write_error)?;
                bytes_transferred_since_update += chunk.len() as u64;
                let elapsed = last_update_time.elapsed();
                if elapsed >= update_interval {
                    speed =
                        ((bytes_transferred_since_update as f64) / elapsed.as_secs_f64()) as u64;
                    progress += bytes_transferred_since_update;
                    match tx
                        .send(DownloadProgress {
                            progress,
                            download_size,
                            speed,
                            time_elapsed: start_time.elapsed(),
                            finished: None,
                            thread_id,
                        })
                        .await
                    {
                        Ok(_) => {}
                        Err(err) => {
                            error!("Error sending progress: {}", err);
                        }
                    }

                    last_update_time = std::time::Instant::now();
                    bytes_transferred_since_update = 0;
                    chunk_sizes.clear();
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

    progress += bytes_transferred_since_update;

    let average_chunk_size = chunk_sizes.iter().sum::<f64>() / chunk_sizes.len() as f64;

    match tx
        .send(DownloadProgress {
            progress,
            download_size,
            speed: (progress as f64 / start_time.elapsed().as_secs_f64()) as u64,
            time_elapsed: start_time.elapsed(),
            finished: Some(DownloadFinished {
                completed: progress == download_size,
                error: false,
                average_chunk_size,
            }),
            thread_id,
        })
        .await
    {
        Ok(_) => {}
        Err(err) => {
            error!("Error sending progress: {}", err);
        }
    }

    Ok(())
}
