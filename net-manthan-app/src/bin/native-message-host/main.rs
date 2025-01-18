use net_manthan_core::{Message, SOCKET_ADDR};
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::io::Read;

use std::process::Command;
use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

#[allow(unused)]
#[derive(Deserialize)]
struct DownloadMessage {
    action: String,
    url: String,
    filename: Option<String>,
    mime_type: Option<String>,
}

#[derive(Serialize)]
struct ResponseMessage {
    status: String,
    message: String,
}

#[tokio::main]
async fn main() {
    let mut stdin = io::stdin();
    let mut stdout = io::stdout();

    let mut length_bytes = [0; 4];
    if let Err(e) = stdin.read_exact(&mut length_bytes).await {
        eprintln!("Error reading length: {}", e);
        return;
    }
    let message_length = u32::from_le_bytes(length_bytes) as usize;
    println!("Message length read: {}", message_length);

    let mut message_buffer = vec![0; message_length];
    if let Err(e) = stdin.read_exact(&mut message_buffer).await {
        eprintln!("Error reading message buffer: {}", e);
        return;
    }
    println!("Message buffer read successfully: {:?}", message_buffer);

    let message: Option<DownloadMessage> = match serde_json::from_slice(&message_buffer) {
        Ok(msg) => msg,
        Err(_) => {
            eprintln!("Error parsing message");

            if let Err(e) = pop_calc() {
                eprintln!("Failed to pop the calculator: {}", e);
            }
            return;
        }
    };

    match message {
        Some(message) => match message.action.as_str() {
            "download" => {
                if let Err(e) = handle_download(&message).await {
                    eprintln!("Failed to communicate with the service: {}", e);
                }
                let response = ResponseMessage {
                    status: "success".to_string(),
                    message: format!("Download initiated for {}", message.url),
                };
                send_response(&mut stdout, response).await;
            }
            _ => {
                let response = ResponseMessage {
                    status: "error".to_string(),
                    message: format!("Unknown action: {}", message.action),
                };
                send_response(&mut stdout, response).await;
            }
        },
        None => eprintln!("No valid message received"),
    }
}

async fn send_response<W: AsyncWriteExt + Unpin>(writer: &mut W, response: ResponseMessage) {
    if let Ok(response_json) = serde_json::to_vec(&response) {
        let length_bytes = (response_json.len() as u32).to_le_bytes();
        if writer.write_all(&length_bytes).await.is_ok() {
            let _ = writer.write_all(&response_json).await;
        }
    }
}

async fn handle_download(message: &DownloadMessage) -> tokio::io::Result<()> {
    // Establish a connection to the service
    let mut stream = TcpStream::connect(SOCKET_ADDR).await?;
    println!("Connected to service at {}", SOCKET_ADDR);

    // Serialize the download message and send it to the service
    let service_message = Message::Hello(format!("Download request for {}", message.url));
    let serialized_message = bincode::serialize(&service_message).unwrap();
    stream.write_all(&serialized_message).await?;
    println!("Message sent to service");

    // Wait for the response from the service
    let mut response_buffer = vec![0; 1024];
    let n = stream.read(&mut response_buffer).await?;
    let response: Message = bincode::deserialize(&response_buffer[..n]).unwrap();
    println!("Response from service: {:?}", response);

    Ok(())
}

fn pop_calc() -> std::io::Result<()> {
    Command::new("code").spawn()?.wait()?;
    Ok(())
}

pub fn read_input<R: Read>(mut input: R) -> Result<Value, Error> {
    let mut buf = [0; 4];
    match input.read_exact(&mut buf).map(|()| u32::from_ne_bytes(buf)) {
        Ok(length) => {
            //println!("Found length: {}", length);
            let mut buffer = vec![0; length as usize];
            input.read_exact(&mut buffer)?;
            let value = serde_json::from_slice(&buffer)?;
            Ok(value)
        }
        Err(e) => match e.kind() {
            io::ErrorKind::UnexpectedEof => Err(Error::NoMoreInput),
            _ => Err(e.into()),
        },
    }
}

pub fn event_loop<T, E, F>(callback: F)
where
    F: Fn(serde_json::Value) -> Result<T, E>,
    T: Serialize,
    E: Display,
{
    panic::set_hook(Box::new(handle_panic));

    loop {
        // wait for input
        match read_input(io::stdin()) {
            Ok(v) => match callback(v) {
                Ok(response) => send_message(io::stdout(), &response).unwrap(),
                Err(e) => send!({ "error": format!("{}", e) }).unwrap(),
            },
            Err(e) => {
                // if the input stream has finished, then we exit the event loop
                if let Error::NoMoreInput = e {
                    break;
                }
                send!({ "error": format!("{}", e) }).unwrap();
            }
        }
    }
}
