use std::path::PathBuf;

use chrome_native_messaging::event_loop;
use net_manthan_core::types::{IpcRequest, IpcResponse};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use utils::{Client, IPC_SOCKET_ADDRESS};

#[derive(Serialize)]
struct ResponseToExtension {
    payload: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DownloadRequest {
    pub url: String,
    pub filename: String,
    pub mime: Option<String>,
    pub referrer: Option<String>,
    pub headers: Option<Vec<String>>,
}

fn main() {
    event_loop(|value| match value {
        Value::Null => Err("null payload"),
        Value::Object(request) => {
            let mut client = match Client::new(IPC_SOCKET_ADDRESS) {
                Ok(client) => client,
                Err(_) => {
                    return Ok(ResponseToExtension {
                        payload: "Could not connect to the server".to_string(),
                    });
                }
            };
            let download_request: DownloadRequest =
                match serde_json::from_value(Value::Object(request)) {
                    Ok(request) => request,
                    Err(err) => {
                        return Ok(ResponseToExtension {
                            payload: format!("Invalid request, {}", err),
                        });
                    }
                };

            let message = IpcRequest::StartDownload {
                url: download_request.url,
                output_path: Some(PathBuf::from(download_request.filename)),
                thread_count: None,
                headers: download_request.headers,
            };

            match client.send_and_receive(message) {
                Ok(response) => {
                    return match response {
                        IpcResponse::Success => Ok(ResponseToExtension {
                            payload: "Download Started".to_string(),
                        }),
                        _ => Ok(ResponseToExtension {
                            payload: "Something went wrong".to_string(),
                        }),
                    };
                }
                Err(e) => {
                    return Ok(ResponseToExtension {
                        payload: format!("Could not send the request. ERR: {}", e),
                    });
                }
            }
        }
        _ => Err("invalid payload"),
    });
}
