use std::fmt;

#[derive(Debug)]
pub enum DownloadError {
    InvalidRequest(String),
    DownloadError(String),
    DownloadInfoFetchError(String),
    DownloadPartError(String),
    FileError(String),
}

impl fmt::Display for DownloadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DownloadError::InvalidRequest(msg) => write!(f, "Invalid Request: {}", msg),
            DownloadError::DownloadError(msg) => write!(f, "Network Error: {}", msg),
            DownloadError::DownloadInfoFetchError(msg) => {
                write!(f, "Failed to fetch download info: {}", msg)
            }
            DownloadError::DownloadPartError(msg) => {
                write!(f, "Failed to download part: {}", msg)
            }
            DownloadError::FileError(msg) => write!(f, "File Error: {}", msg),
        }
    }
}

impl std::error::Error for DownloadError {}
