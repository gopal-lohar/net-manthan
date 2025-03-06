use crate::errors::DownloadError;
use crate::types::DownloadRequest;
use reqwest::{header, Client, Url};

pub struct DownloadInfo {
    pub size: u64,
    pub resume: bool,
    pub filename: Option<String>,
}

// TODO: fix the multithreading issue: it splits the downloads even if it doesn't support resuming
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

    // TODO: it gives wrong names!!! url encoding issue and other weird stuff
    // TODO: 1. Windows11InstallationAssistant.exe; filename*=UTF-8''Windows11InstallationAssistant.exe
    // TODO: 2. Docker%20Desktop%20Installer.exe
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
