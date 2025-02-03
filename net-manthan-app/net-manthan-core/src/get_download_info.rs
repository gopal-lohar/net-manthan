use crate::types::DownloadRequest;
use crate::errors::DownloadError;
use reqwest::{header, Client};

pub struct DownloadInfo {
  pub   size: u64,
  pub   resume: bool,
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

    Ok(DownloadInfo { size, resume })
}
