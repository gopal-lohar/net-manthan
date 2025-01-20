use colored::*;
use futures_util::StreamExt;
use reqwest::{header, Client};
use serde::Deserialize;
use std::fs::{File, OpenOptions};
use std::io::{self, Seek, SeekFrom, Write};
use tokio::task;
use tracing::{error, info};

#[derive(Debug, Deserialize, Clone)]
pub struct DownloadRequest {
    url: String,
    filename: String,
    mime: Option<String>,
    referrer: Option<String>,
    headers: Option<Vec<String>>,
}

struct DownloadPart {
    start: u64,
    end: u64,
    part: usize,
}

async fn download_part(
    client: Client,
    url: String,
    part: DownloadPart,
    file: &mut File,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let range_header = format!("bytes={}-{}", part.start, part.end);

    let response = client
        .get(&url)
        .header(header::RANGE, range_header)
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(Box::new(io::Error::new(
            io::ErrorKind::Other,
            format!("Failed to download part {}", part.part),
        )));
    }

    let mut stream = response.bytes_stream();
    let mut position = part.start;

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        file.seek(SeekFrom::Start(position))?;
        file.write_all(&chunk)?;
        position += chunk.len() as u64;

        file.flush()?;
    }

    Ok(())
}

pub async fn handle_download(request: DownloadRequest) -> Result<(), Box<dyn std::error::Error>> {
    info!("Download request received: {:?}", request);

    let client = Client::new();
    let response = client
        .get(&request.url)
        .header(header::RANGE, "bytes=0-0")
        .send()
        .await?;

    let supports_partial = response.headers().get(header::ACCEPT_RANGES).is_some();
    if !supports_partial {
        error!("Server doesn't support partial downloads");
        return Err(Box::new(io::Error::new(
            io::ErrorKind::Other,
            "Server doesn't support partial downloads",
        )));
    }

    let total_size = response
        .headers()
        .get(header::CONTENT_LENGTH)
        .and_then(|val| val.to_str().ok())
        .and_then(|val| val.parse::<u64>().ok())
        .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Couldn't get file size"))?;

    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&request.filename)?;

    file.set_len(total_size)?;

    const PART_COUNT: u64 = 5;
    let part_size = total_size / PART_COUNT;
    let mut tasks = Vec::new();

    for i in 0..PART_COUNT {
        let start = i * part_size;
        let end = if i == PART_COUNT - 1 {
            total_size - 1
        } else {
            start + part_size - 1
        };

        let part = DownloadPart {
            start,
            end,
            part: i as usize,
        };

        let client = client.clone();
        let url = request.url.clone();
        let mut file = OpenOptions::new().write(true).open(&request.filename)?;

        let handle = task::spawn(async move { download_part(client, url, part, &mut file).await });

        tasks.push(handle);
    }

    let stdout = io::stdout();
    let mut handle = stdout.lock();
    write!(
        handle,
        "\r\tStarting concurrent download with {} parts",
        PART_COUNT
    )
    .unwrap();
    handle.flush().unwrap();

    for task in tasks {
        task.await??;
    }

    writeln!(handle, "\n\tDownload completed!").unwrap();
    Ok(())
}
