use crate::errors::DownloadError;
use crate::types::DownloadRequest;
use reqwest::{header, Client, Url};

pub struct DownloadInfo {
    pub size: u64,
    pub resume: bool,
    pub filename: Option<String>,
}

pub async fn get_download_info(request: &DownloadRequest) -> Result<DownloadInfo, DownloadError> {
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

    let filename = response
        .headers()
        .get(header::CONTENT_DISPOSITION)
        .and_then(|val| val.to_str().ok())
        .and_then(|val| {
            val.split("filename=")
                .nth(1)
                .map(|s| s.trim_matches('"').to_string())
        })
        .or_else(|| {
            Url::parse(&request.url)
                .ok()
                .and_then(|url| url.path_segments()?.last().map(String::from))
        });

    Ok(DownloadInfo {
        size,
        resume,
        filename,
    })
}
