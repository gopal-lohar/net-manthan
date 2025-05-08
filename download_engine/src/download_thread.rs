use std::sync::Arc;

use crate::{
    Download, DownloadParts, DownloadPartsProgress, DownloadProgressPart, errors::DownloadError,
    open_file_writer::open_file_writer, types::DownloadStatus,
};
use chrono::Utc;
use futures_util::StreamExt;
use reqwest::{
    Client,
    header::{self, HeaderMap, HeaderName, HeaderValue},
};
use tokio::{io::AsyncWriteExt, sync::Mutex};
use tracing::{error, info, trace};

impl Download {
    pub async fn start(&mut self) -> Result<(), DownloadError> {
        match &mut self.parts {
            DownloadParts::NonResumable(part) => part.status = DownloadStatus::Connecting,
            DownloadParts::Resumable(parts) => {
                for part in parts {
                    part.status = DownloadStatus::Connecting;
                }
            }
            DownloadParts::None => {}
        }
        match &self.progress {
            DownloadPartsProgress::NonResumable(part) => {
                let me = self.clone();
                let part = DownloadProgressPart::NonResumable(part.clone());
                tokio::spawn(async move {
                    match me.download(part).await {
                        Ok(_) => {
                            // info!("Download completed")
                        }
                        Err(e) => error!("Download failed: {}", e),
                    }
                });
            }
            DownloadPartsProgress::Resumable(parts) => {
                parts.iter().for_each(|part| {
                    let me = self.clone();
                    let part = DownloadProgressPart::Resumable(part.clone());
                    tokio::task::spawn(async move {
                        match me.download(part).await {
                            Ok(_) => {
                                // info!("Download completed")
                            }
                            Err(e) => error!("Download failed: {}", e),
                        }
                    });
                });
            }
            DownloadPartsProgress::None => {
                return Err(DownloadError::GeneralError(
                    "download info not loaded".into(),
                ));
            }
        }
        Ok(())
    }

    async fn download(self, part: DownloadProgressPart) -> Result<(), DownloadError> {
        let last_flush_time = Arc::new(Mutex::new(Utc::now()));

        let client = Client::new();
        let mut req = client.get(&self.url);

        if let Some(headers) = &self.headers {
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

        match &part {
            DownloadProgressPart::Resumable(part) => {
                let part = part.lock().await;
                req = req.header(
                    header::RANGE,
                    format!(
                        "bytes={}-{}",
                        part.start_byte + part.bytes_downloaded,
                        part.end_byte
                    ),
                )
            }
            DownloadProgressPart::NonResumable(_) => {}
        }

        let part_clone = part.clone();
        let mut writer = match open_file_writer(
            self.file.clone(),
            match &part {
                DownloadProgressPart::Resumable(part) => part.lock().await.bytes_downloaded,
                DownloadProgressPart::NonResumable(_) => 0,
            },
            self.config.buffer_size,
            // this is gymnastics, probably just lagging 1 * buffer in progress would have been better
            // can't make the closure async so i have to spawn a new tokio task
            // TODO: This need to be fixed!!!!
            // there is no way we can insert a on_flush on an existing bufwriter easily
            // the only way is to write my own, very own simple async bufwriter
            Box::new(move |bytes_flushed| {
                let bytes_flushed = bytes_flushed;
                let last_flush_time = last_flush_time.clone();
                let part = part_clone.clone();
                tokio::spawn(async move {
                    let mut last_flush_time = last_flush_time.lock().await;
                    let current_time = Utc::now();
                    let time_elapsed = current_time - *last_flush_time;
                    let current_speed =
                        ((bytes_flushed as f64) / time_elapsed.as_seconds_f64()) as usize;
                    *last_flush_time = current_time;
                    match part {
                        DownloadProgressPart::Resumable(part) => {
                            let mut part = part.lock().await;
                            part.bytes_downloaded += bytes_flushed as u64;
                            part.current_speed = current_speed;
                            if part.bytes_downloaded == (*part).get_total_size() {
                                part.status = DownloadStatus::Complete;
                                info!(
                                    "Download Finished {}/{}",
                                    part.bytes_downloaded,
                                    part.get_total_size()
                                );
                                info!("bytes_flushed: {}", bytes_flushed);
                            }
                        }
                        DownloadProgressPart::NonResumable(part) => {
                            let mut part = part.lock().await;
                            part.bytes_downloaded += bytes_flushed as u64;
                            part.current_speed = current_speed;
                            if part.bytes_downloaded == (*part).total_size {
                                part.status = DownloadStatus::Complete;
                            }
                        }
                    }
                });
            }),
        )
        .await
        {
            Ok(writer) => writer,
            Err(err) => return Err(DownloadError::FileSystemError(err)),
        };

        let response = match req.send().await {
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

        match &part {
            DownloadProgressPart::NonResumable(part) => {
                part.lock().await.status = DownloadStatus::Downloading;
            }
            DownloadProgressPart::Resumable(parts) => {
                parts.lock().await.status = DownloadStatus::Downloading;
            }
        }

        let mut stream = response.bytes_stream();

        while let Some(chunk) = stream.next().await {
            match chunk {
                Ok(chunk) => {
                    writer.write_all(&chunk).await?;
                }
                Err(err) => {
                    return Err(DownloadError::HttpRequestError(err));
                }
            }
        }

        writer.flush().await?;

        Ok(())
    }
}
