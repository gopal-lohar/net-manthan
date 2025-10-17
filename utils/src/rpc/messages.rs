use std::usize;

use bincode::{
    config,
    error::{DecodeError, EncodeError},
    serde::{decode_from_slice, encode_to_vec},
};
use chrono::{DateTime, Utc};
use engine::{
    helpers::random::generate_random_id,
    types::{download::Download, messages::DownloadRequestMessage, request::DownloadInfo},
};
use serde::{Deserialize, Serialize};
const BINCODE_CONFIG: config::Configuration = config::standard();

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    // id of the request, it should be present in both request and response
    pub request_id: i64,
    pub secret: String,
    pub timestamp: DateTime<Utc>,
    pub payload: Payload,
}

impl Message {
    pub fn request(request: RpcRequest, secret: String) -> Self {
        Self {
            request_id: generate_random_id(),
            secret,
            timestamp: Utc::now(),
            payload: Payload::Request(request),
        }
    }

    pub fn response(request_id: i64, secret: String, response: RpcResponse) -> Self {
        Self {
            request_id,
            secret,
            timestamp: Utc::now(),
            payload: Payload::Response(response),
        }
    }

    pub fn new_response(secret: String, response: RpcResponse) -> Self {
        Self {
            request_id: generate_random_id(),
            secret,
            timestamp: Utc::now(),
            payload: Payload::Response(response),
        }
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, DecodeError> {
        let (message, _): (Message, usize) = decode_from_slice(&bytes, BINCODE_CONFIG)?;
        Ok(message)
    }

    pub fn encode(&self) -> Result<Vec<u8>, EncodeError> {
        encode_to_vec(&self, BINCODE_CONFIG)
    }
}

// random but static bytes meaning encoding/decoding errors
pub const MAGIC_RESPONSE: &[u8] = &[7u8, 3u8, 5u8, 7u8, 9u8, 0u8, 2u8, 1u8, 6u8, 9u8];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Payload {
    Request(RpcRequest),
    Response(RpcResponse),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RpcRequest {
    Heartbeat,
    DownloadRequest(DownloadRequestMessage),
    SetDownloadInfo((i64, Result<DownloadInfo, String>)),
    GetDownload(String),
    GetDownloads(Vec<String>),
    PauseDownload(i64),
    PauseDownloads(Vec<String>),
    ResumeDownload(i64),
    ResumeDownloads(Vec<String>),
    Shutdown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RpcResponse {
    Heartbeat,
    DownloadStarted(String),
    Success,
    Recieved,
    RequestNotSupported,
    Error(String),
    Download(Download),
    Downloads(Vec<Download>),
}
