use colored::*;
use futures_util::StreamExt;
use reqwest::{header, Client};
use serde::Deserialize;
use std::fs::OpenOptions;
use std::io::{self, Write};
use tracing::{error, info};

#[derive(Debug, Deserialize)]
pub struct DownloadRequest {
    url: String,
    filename: String,
    mime: Option<String>,
    referrer: Option<String>,
    headers: Option<Vec<String>>,
}

pub async fn handle_download(request: DownloadRequest) -> Result<(), Box<dyn std::error::Error>> {
    info!("Download request received: {:?}", request);

    info!("Starting download");
    info!("Downloading file from: {}", request.filename);

    if let Some(mime) = request.mime {
        info!("\tMIME: {}", mime);
    }
    if let Some(referrer) = request.referrer {
        info!("\tReferrer: {}", referrer);
    }
    if let Some(headers) = request.headers {
        for header in headers {
            info!("\t\tHeader: {}", header);
        }
    }

    let client = Client::new();
    let response = client.get(request.url.clone()).send().await?;

    if !response.status().is_success() {
        error!("Failed to download file: {}", response.status());
        return Err(Box::new(io::Error::new(
            io::ErrorKind::Other,
            "Failed to download file",
        )));
    }

    // for (key, value) in response.headers() {
    //     println!("{}: {}", key.to_string().yellow(), value.to_str()?);
    // }

    let total_size = response
        .headers()
        .get(header::CONTENT_LENGTH)
        .and_then(|val| val.to_str().ok())
        .and_then(|val| val.parse::<u64>().ok());

    if total_size.is_none() {
        error!("Failed to determine file size");
    }

    let total_size = total_size.unwrap_or(1);

    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(request.filename)?;

    let mut stream = response.bytes_stream();
    println!("");
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    let mut downloaded = 0;

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        file.write_all(&chunk)?;
        file.flush()?;

        downloaded += chunk.len() as u64;
        let progress = (downloaded as f64 / total_size as f64) * 100.0;

        write!(
            handle,
            "\r\tProgress: [{}] [{}{}]",
            format!("{:3.0}%", progress).to_string().green(),
            "#".repeat((progress / 2.0) as usize).green(),
            " ".repeat(50 - (progress / 2.0) as usize)
        )
        .unwrap();
        handle.flush().unwrap();
    }
    writeln!(handle).unwrap();
    println!("");

    Ok(())
}
