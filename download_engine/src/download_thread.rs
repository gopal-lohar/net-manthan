use crate::{
    Download, DownloadPart, DownloadParts, errors::DownloadError,
    open_file_writer::open_file_writer, utils::format_speed,
};
use futures_util::StreamExt;
use reqwest::{
    Client,
    header::{self, HeaderMap, HeaderName, HeaderValue},
};
use tokio::io::AsyncWriteExt;
use tracing::{error, info, trace};

impl Download {
    pub async fn start(&self) -> Result<(), DownloadError> {
        match &self.parts {
            DownloadParts::NonResumable(part) => {
                let me = self.clone();
                let part = DownloadPart::NonResumable(part.clone());
                tokio::spawn(async move {
                    match me.download(part).await {
                        Ok(_) => info!("Download completed"),
                        Err(e) => error!("Download failed: {}", e),
                    }
                });
            }
            DownloadParts::Resumable(parts) => {
                parts.iter().for_each(|part| {
                    let me = self.clone();
                    let part = DownloadPart::Resumable(part.clone());
                    tokio::task::spawn(async move {
                        match me.download(part).await {
                            Ok(_) => info!("Download completed"),
                            Err(e) => error!("Download failed: {}", e),
                        }
                    });
                });
            }
            DownloadParts::None => {}
        }
        Ok(())
    }

    async fn download(self, part: DownloadPart) -> Result<(), DownloadError> {
        trace!("Opening writer");
        let mut writer = match open_file_writer(
            self.file.clone(),
            match &part {
                DownloadPart::Resumable(part) => part.bytes_downloaded,
                DownloadPart::NonResumable(_) => 0,
            },
            self.config.buffer_size,
            Box::new(move |bytes_flushed| {
                info!(
                    "Downloaded {} bytes which is = {}",
                    bytes_flushed,
                    format_speed(bytes_flushed as u64)
                );
            }),
        )
        .await
        {
            Ok(writer) => writer,
            Err(err) => return Err(DownloadError::FileSystemError(err)),
        };

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
            DownloadPart::Resumable(part) => {
                req = req.header(
                    header::RANGE,
                    format!(
                        "bytes={}-{}",
                        part.start_byte + part.bytes_downloaded,
                        part.end_byte
                    ),
                )
            }
            DownloadPart::NonResumable(_) => {}
        }

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
