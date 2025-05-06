use crate::{Download, DownloadPart, errors::DownloadError, open_file_writer::open_file_writer};
use futures_util::StreamExt;
use reqwest::{
    Client,
    header::{self, HeaderMap, HeaderName, HeaderValue},
};
use tokio::io::AsyncWriteExt;

impl Download {
    pub async fn start(&self) -> Result<(), DownloadError> {
        Ok(())
    }

    async fn download(&self, part: DownloadPart) -> Result<(), DownloadError> {
        let mut writer = match open_file_writer(
            self.file.clone(),
            match &part {
                DownloadPart::Resumable(part) => part.bytes_downloaded,
                DownloadPart::NonResumable(_) => 0,
            },
            move || {},
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

        Ok(())
    }
}
