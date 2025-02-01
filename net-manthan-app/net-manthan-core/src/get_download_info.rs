use crate::types::DownloadRequest;
use crate::{download_part::DownloadPart, errors::DownloadError};
use reqwest::{header, Client};

struct DownloadInfo {
    size: u64,
    resume: bool,
}

async fn check_download_info(request: &DownloadRequest) -> Result<DownloadInfo, DownloadError> {
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

pub async fn get_download_info(
    download_id: u64,
    thread_count: usize,
    request: &DownloadRequest,
) -> Result<(Vec<DownloadPart>, u64), DownloadError> {
    let mut parts = Vec::<DownloadPart>::new();
    let download_info = check_download_info(&request).await?;
    let thread_count = match download_info.resume {
        true => thread_count,
        false => 1,
    };
    for i in 0..thread_count {
        let part = DownloadPart {
            download_id,
            chunk_id: i as u32,
            url: request.url.clone(),
            filepath: request.filename.clone(),
            bytes_downloaded: 0,
            range: match download_info.resume {
                true => Some((
                    (download_info.size as f64 / thread_count as f64 * i as f64).round() as u64,
                    (download_info.size as f64 / thread_count as f64 * (i + 1) as f64).round()
                        as u64,
                )),
                false => None,
            },
        };
        parts.push(part);
    }
    Ok((parts, download_info.size))
}
