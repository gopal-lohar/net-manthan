use chrome_native_messaging::event_loop;
use serde::Serialize;
use serde_json::Value;
use std::{io::Write, os::unix::net::UnixStream};

#[derive(Serialize)]
struct BasicMessage<'a> {
    payload: &'a str,
}

const SOCKET_PATH: &str = "/tmp/net-manthan.sock";

fn main() {
    event_loop(|value| match value {
        Value::Null => Err("null payload"),
        Value::Object(request) => {
            let request = match serde_json::to_string(&request) {
                Ok(request) => request.as_bytes().to_vec(),
                Err(_) => return Err("invalid payload"),
            };

            let mut stream = match UnixStream::connect(SOCKET_PATH) {
                Ok(stream) => stream,
                Err(_) => {
                    return Ok(BasicMessage {
                        payload: "colud not connect to the daemon",
                    })
                }
            };

            if let Err(_) = stream.write_all(&request) {
                return Ok(BasicMessage {
                    payload: "could not send the request",
                });
            }

            return Ok(BasicMessage {
                payload: "Download request sent",
            });
        }
        _ => Err("invalid payload"),
    });
}
