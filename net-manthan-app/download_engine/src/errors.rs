use reqwest;
use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DownloadError {
    /// An error occurred while making an HTTP request.
    #[error("HTTP request failed: {0}")]
    HttpRequestError(#[from] reqwest::Error),

    /// The content length of the file is unknown.
    #[error("Content length is unknown, unable to proceed")]
    UnknownContentLength,

    /// Failed to create or access the output file.
    #[error("File system error: {0}")]
    FileSystemError(#[from] io::Error),

    /// The download was interrupted unexpectedly.
    #[error("Download interrupted")]
    DownloadInterrupted,

    /// Error occurred while writing to the file.
    #[error("Write error: {0}")]
    WriteError(String),

    /// General error for unexpected scenarios.
    #[error("Unexpected error: {0}")]
    GeneralError(String),
}

impl DownloadError {
    /// Creates a `DownloadError::WriteError` from an I/O error.
    pub fn from_write_error(err: io::Error) -> Self {
        DownloadError::WriteError(err.to_string())
    }

    /// Create a `DownloadError::GeneralError` with a custom message.
    pub fn general(msg: impl Into<String>) -> Self {
        DownloadError::GeneralError(msg.into())
    }
}
