use net_manthan_core::{Message, SOCKET_ADDR};
use serde::Deserialize;
use std::process::Command;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

#[allow(unused)]
#[derive(Deserialize)]
struct DownloadMessage {
    action: String,
    url: String,
    filename: Option<String>,
    mime_type: Option<String>,
}

#[tokio::main]
async fn main() {
    let message: DownloadMessage = DownloadMessage {
        action: "download".to_string(),
        url: "https://www.rust-lang.org".to_string(),
        filename: None,
        mime_type: None,
    };
    if let Err(e) = handle_download(&message).await {
        eprintln!("Failed to communicate with the service: {}", e);
    }
}

async fn handle_download(message: &DownloadMessage) -> tokio::io::Result<()> {
    if let Err(e) = pop_calc() {
        eprintln!("Failed to pop the calculator: {}", e);
    }
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
