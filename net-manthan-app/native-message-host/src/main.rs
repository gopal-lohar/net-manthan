use chrome_native_messaging::event_loop;
use net_manthan_core::types::Message;
use serde::Serialize;
use serde_json::Value;
use utils::{Client, IPC_SOCKET_ADDRESS};

#[derive(Serialize)]
struct ResponseToExtension {
    payload: String,
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
            let download_request: Message = match serde_json::from_value(Value::Object(request)) {
                Ok(request) => Message::DownloadRequest(request),
                Err(err) => {
                    return Ok(ResponseToExtension {
                        payload: format!("Invalid request, {}", err),
                    });
                }
            };
            match client.send_and_receive(download_request) {
                Ok(response) => {
                    return match response {
                        Message::DownnloadResponse(response) => {
                            Ok(ResponseToExtension { payload: response })
                        }
                        _ => Ok(ResponseToExtension {
                            payload: "Invalid response".to_string(),
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
