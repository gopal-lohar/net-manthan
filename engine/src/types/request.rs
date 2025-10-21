use crate::{
    APP_NAME_LOWERCASE,
    helpers::{extract_filename::extract_filename, random::generate_random_id},
};
use mime::Mime;
use reqwest::{
    Client, Url,
    header::{ACCEPT_RANGES, AUTHORIZATION, CONTENT_LENGTH, COOKIE, REFERER, USER_AGENT},
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tracing::debug;

use super::errors::DownloadError;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DownloadRequest {
    pub url: String,
    pub directory: String,
    pub rename: Option<String>,
    pub headers: Headers,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Headers {
    pub user_agent: Option<String>,
    pub authorization: Option<String>,
    pub referer: Option<String>,
    pub cookie: Option<String>,
}

impl Headers {
    pub fn none() -> Self {
        Headers {
            user_agent: None,
            authorization: None,
            referer: None,
            cookie: None,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DownloadInfo {
    pub size: Option<u64>,
    pub mime_type: String,
    pub file_name: String,
    pub resumable: bool,
}

impl DownloadRequest {
    pub fn get_file_path(&self, info: &DownloadInfo) -> PathBuf {
        let file_name = match &self.rename {
            Some(name) => name.clone(),
            None => info.file_name.clone(),
        };
        let mut path = PathBuf::from(&self.directory);
        path.push(file_name);
        path
    }

    pub fn get_file_path_str(&self, info: &DownloadInfo) -> String {
        self.get_file_path(info).to_string_lossy().to_string()
    }

    pub fn is_valid_http_url(s: &str) -> bool {
        match Url::parse(s) {
            Ok(url) => url.scheme() == "http" || url.scheme() == "https",
            Err(_) => false,
        }
    }

    pub fn build_request_headers(
        &self,
        mut request: reqwest::RequestBuilder,
    ) -> reqwest::RequestBuilder {
        // Set headers if present
        if let Some(ua) = &self.headers.user_agent {
            request = request.header(USER_AGENT, ua);
        }
        if let Some(auth) = &self.headers.authorization {
            request = request.header(AUTHORIZATION, auth);
        }
        if let Some(refr) = &self.headers.referer {
            request = request.header(REFERER, refr);
        }
        if let Some(ck) = &self.headers.cookie {
            request = request.header(COOKIE, ck);
        }
        request
    }

    pub fn build_request(&self, client: &reqwest::Client) -> reqwest::RequestBuilder {
        let mut request = client.get(&self.url);
        request = self.build_request_headers(request);
        request
    }

    // just give you back download info for a requeset
    pub async fn load_download_info(&self) -> Result<DownloadInfo, DownloadError> {
        let client = Client::new();
        let response = self.build_request(&client).send().await?;

        let size = response
            .headers()
            .get(CONTENT_LENGTH)
            .and_then(|val| val.to_str().ok())
            .and_then(|val| val.parse::<u64>().ok());

        let resumable = response.headers().get(ACCEPT_RANGES).is_some();

        // if the request doesn't provide a filename, we try to get it from
        // the response headers
        // or from the URL
        // or a fallback name using a random number (not related to download id)
        let file_name = match &self.rename {
            Some(name) => name.into(),
            None => match extract_filename(response.headers(), &self.url) {
                Some(name) => name,
                None => Self::get_random_file_name(),
            },
        };

        let mime_type = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .and_then(|s| s.parse::<Mime>().ok())
            .unwrap_or(mime::APPLICATION_OCTET_STREAM);

        debug!(
            "the download with link {:?} is resumable: {} mime_type: {:?}",
            self.url, resumable, mime_type
        );

        Ok(DownloadInfo {
            mime_type: mime_type.to_string(),
            file_name,
            size,
            resumable,
        })
    }

    pub fn get_random_file_name() -> String {
        format!("{}_download_{}", APP_NAME_LOWERCASE, generate_random_id())
    }
}
