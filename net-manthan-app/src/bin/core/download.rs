use colored::*;
use futures_util::StreamExt;
use reqwest::{header, Client};
use serde::Deserialize;
use std::fs::{File, OpenOptions};
use std::io::{self, Write};
use std::sync::{Arc, Mutex};
use tracing::{error, info};

#[derive(Clone, Debug, Deserialize)]
pub struct DownloadRequest {
    url: String,
    filename: String,
    mime: Option<String>,
    referrer: Option<String>,
    headers: Option<Vec<String>>,
}

#[derive(Clone, Debug)]
struct DownloadTaskPart {
    url: String,
    start: u64,
    end: u64,
    downloaded: u64,
    part_id: u64,
}

struct DownloadTask {
    size: u64,
    threads: u64,
    parts: Vec<DownloadTaskPart>,
}

struct DownloadInfo {
    size: u64,
    supports_parts: bool,
}

async fn check_download_info(
    request: &DownloadRequest,
    client: &Client,
) -> Result<DownloadInfo, Box<dyn std::error::Error>> {
    let response = client.get(&request.url).send().await?;

    // Extract and parse the Content-Length header
    let content_length = response
        .headers()
        .get(header::CONTENT_LENGTH)
        .and_then(|val| val.to_str().ok())
        .and_then(|val| val.parse::<u64>().ok())
        .unwrap_or(0);

    Ok(DownloadInfo {
        size: content_length,
        supports_parts: response.headers().get(header::ACCEPT_RANGES).is_some(),
    })
}

async fn download_part(
    part_id: u64,
    client: Client,
    url: String,
    download_task: Arc<Mutex<DownloadTask>>,
    file: &mut File,
) -> Result<(), String> {
    let part = download_task.lock().unwrap().parts[part_id as usize].clone();
    let range_header = format!("bytes={}-{}", part.start, part.end);

    let response = match client
        .get(&url)
        .header(header::RANGE, range_header)
        .send()
        .await
    {
        Ok(response) => response,
        Err(_) => return Err("oh we couldn't download the thread".to_string()),
    };

    if !response.status().is_success() {
        return Err("oh we couldn't download the thread".to_string());
    }

    let mut stream = response.bytes_stream();
    let mut i = 0;
    while let Some(chunk) = stream.next().await {
        let chunk = match chunk {
            Ok(chunk) => chunk,
            Err(_) => return Err("oh we couldn't download the thread 1.".to_string()),
        };
        match file.write_all(&chunk) {
            Ok(_) => (),
            Err(_) => return Err("oh we couldn't download the thread 2.".to_string()),
        }

        download_task.lock().unwrap().parts[part_id as usize].downloaded += chunk.len() as u64;

        match file.flush() {
            Ok(_) => (),
            Err(_) => return Err("oh we couldn't download the thread 3.".to_string()),
        }
        info!("downloading index: {} in the thread {}", i, part_id);
        i += 1;
    }

    Ok(())
}

pub async fn handle_download(request: DownloadRequest) -> Result<(), Box<dyn std::error::Error>> {
    info!("Download request received: {:?}", request);

    if let Some(mime) = &request.mime {
        info!("\tMIME: {}", mime);
    }
    if let Some(referrer) = &request.referrer {
        info!("\tReferrer: {}", referrer);
    }
    if let Some(headers) = &request.headers {
        for header in headers {
            info!("\t\tHeader: {}", header);
        }
    }

    info!("checking if it support parts");
    let client = Client::new();
    let download_info = check_download_info(&request, &client).await?;
    info!("download size is : {}", download_info.size);
    let download_task = Arc::new(Mutex::new(DownloadTask {
        size: download_info.size,
        threads: 5,
        parts: Vec::new(),
    }));

    if download_info.supports_parts {
        info!("the request does Supports parts!!!");
        let thread_count = download_task.lock().unwrap().threads;
        for i in 0..thread_count {
            let size = download_task.lock().unwrap().size;
            info!("size: {}", size);
            let part_size = size / thread_count as u64;
            let start = i * part_size;
            let end = if i == download_task.lock().unwrap().threads - 1 {
                download_task.lock().unwrap().size - 1
            } else {
                start + part_size - 1
            };

            download_task.lock().unwrap().parts.push(DownloadTaskPart {
                url: request.url.clone(),
                start,
                end,
                downloaded: 0,
                part_id: i,
            });
        }

        for part in download_task.lock().unwrap().parts.iter() {
            info!("part: {:?}", part);
        }

        let mut tasks = Vec::new();

        for part in download_task.lock().unwrap().parts.iter() {
            let client = client.clone();
            let url = part.url.clone();
            let part_id = part.part_id;
            let download_task_clone = Arc::clone(&download_task);
            let mut file = OpenOptions::new()
                .create(true)
                .write(true)
                .open(format!("{}.part{}", &request.filename, part_id,))?;
            let handle = tokio::spawn(async move {
                download_part(part_id, client, url, download_task_clone, &mut file).await
            });
            tasks.push(handle);
        }

        while {
            let mut all_done = true;
            for part in download_task.lock().unwrap().parts.iter() {
                if part.downloaded < part.end - part.start {
                    all_done = false;
                    break;
                }
            }
            !all_done
        } {
            println!("");
            let stdout = io::stdout();
            let mut handle = stdout.lock();

            write!(handle, "\r").unwrap();

            for part in download_task.lock().unwrap().parts.iter() {
                let size = if part.end - part.start > 0 {
                    part.end - part.start
                } else {
                    1
                };
                let progress = (part.downloaded as f64 / size as f64) * 100.0;
                write!(
                    handle,
                    "\r\tProgress: [{}] [{}{}]\n",
                    format!("{:3.0}%", progress).to_string().green(),
                    "#".repeat((progress / 2.0) as usize).green(),
                    " ".repeat(50 - (progress / 2.0) as usize)
                )
                .unwrap();
            }
            handle.flush().unwrap();

            writeln!(handle).unwrap();
            println!("");
            write!(handle, "{}", "\x1B[A".repeat(8)).unwrap();
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        }
    } else {
        info!("oh it Does not support parts");

        info!("Starting solo download");
        info!("Downloading file from: {}", request.filename);

        let response = client.get(request.url.clone()).send().await?;

        if !response.status().is_success() {
            error!("Failed to download file: {}", response.status());
            return Err(Box::new(io::Error::new(
                io::ErrorKind::Other,
                "Failed to download file",
            )));
        }

        let total_size = response
            .headers()
            .get(header::CONTENT_LENGTH)
            .and_then(|val| val.to_str().ok())
            .and_then(|val| val.parse::<u64>().ok());

        match total_size {
            Some(size) => info!("total size is {}", size),
            None => error!("Failed to determine file size"),
        }

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
    }

    Ok(())
}
