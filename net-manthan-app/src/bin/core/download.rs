use crate::errors::DownloadError;
use colored::Colorize;
use futures_util::StreamExt;
use reqwest::{header, Client};
use serde::Deserialize;
use std::fs::{File, OpenOptions};
use std::io::Write;
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

#[derive(Clone, Debug)]
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

async fn download_part(
    client: &Client,
    file: Arc<Mutex<File>>,
    part: DownloadPart,
) -> Result<(), DownloadError> {
    let mut chunk_sizes = Vec::<f64>::new();
    let response = match client.get(&part.url).send().await {
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

    // let mut file = BufWriter::new(file); // I will Create my own buffer

    while let Some(chunk) = stream.next().await {
        match chunk {
            Ok(chunk) => {
                chunk_sizes.push((chunk.len() as f64) / 1024.0);
                // match file.write_all(&chunk) {
                //     Ok(_) => (),
                //     Err(err) => {
                //         return Err(DownloadError::DownloadPartError(format!(
                //             "failed while writing chunk to file(buffer), Error: {}",
                //             err
                //         )));
                //     }
                // }
            }
            Err(err) => {
                return Err(DownloadError::DownloadPartError(format!(
                    "failed while reading chunk from stream, Error: {}",
                    err
                )));
            }
        }
    }

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

    let mut file = Arc::new(Mutex::new(
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
    if download_info.supports_parts {
        info!("Download supports parts");
        let file_clone = Arc::clone(&file);
        let handle = tokio::spawn(async move {
            match download_part(
                &client,
                file_clone,
                DownloadPart {
                    url: request.url,
                    range: None,
                },
            )
            .await
            {
                Ok(_) => (),
                Err(err) => {
                    error!("Failed to download part: {:?}", err);
                }
            }
        });

        match handle.await {
            Ok(_) => (),
            Err(err) => {
                error!("Failed to download part: {:?}", err);
            }
        }
    } else {
        info!("Download does not support parts");
    }

    Ok(())
}
