use std::sync::{Arc, Mutex};

use crate::constants::IPC_SOCKET_ADDRESS;
use crate::download_manager::DownloadManager;
use chrono::Duration;
use net_manthan_core::types::{DownloadRequest, DownloadRequestConfig, IpcRequest, IpcResponse};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tracing::info;

async fn read_message_to_buffer(
    mut buffer: &mut Vec<u8>,
    stream: &mut TcpStream,
) -> std::io::Result<()> {
    buffer.clear();
    // Read the message length (8 bytes for u64)
    let mut len_bytes = [0u8; 8];
    stream.read_exact(&mut len_bytes).await?;
    let msg_len = u64::from_le_bytes(len_bytes) as usize;

    // Read the actual message
    buffer.resize(msg_len, 0);
    stream.read_exact(&mut buffer).await?;
    Ok(())
}

async fn write_message_to_stream(message: Vec<u8>, stream: &mut TcpStream) -> std::io::Result<()> {
    let msg_len = message.len() as u64;
    stream.write_all(&msg_len.to_le_bytes()).await?;
    stream.write_all(&message).await?;
    Ok(())
}

async fn send_response_to_client(
    response: IpcResponse,
    stream: &mut TcpStream,
) -> std::io::Result<()> {
    let response_bytes = match bincode::serialize(&response) {
        Ok(bytes) => bytes,
        Err(_) => return Err(std::io::Error::from(std::io::ErrorKind::InvalidData)),
    };
    write_message_to_stream(response_bytes, stream).await
}

async fn handle_ipc_client(mut stream: TcpStream, download_manager: Arc<Mutex<DownloadManager>>) {
    let mut buffer = Vec::new();
    loop {
        if read_message_to_buffer(&mut buffer, &mut stream)
            .await
            .is_err()
        {
            return;
        } else {
            println!("Message read successfully");
        }

        match bincode::deserialize::<IpcRequest>(&buffer) {
            Ok(request) => match request {
                IpcRequest::HeartBeat => {
                    let response = IpcResponse::HeartBeat;
                    match send_response_to_client(response, &mut stream).await {
                        Ok(_) => {}
                        Err(_) => return,
                    }
                }
                IpcRequest::StartDownload {
                    url,
                    output_path,
                    thread_count,
                    headers,
                } => {
                    info!("Received request");
                    let download_id = 1;
                    download_manager.lock().unwrap().start_download(
                        download_id,
                        DownloadRequest {
                            url,
                            filepath: output_path.unwrap_or("/tmp/test".into()),
                            headers,
                            parts: None,
                            config: DownloadRequestConfig {
                                thread_count: thread_count.unwrap_or(5),
                                buffer_size: 1024 * 1024,
                                update_interval: Duration::seconds(1),
                            },
                        },
                    );
                    println!("Download started");
                    let response = IpcResponse::Success;
                    match send_response_to_client(response, &mut stream).await {
                        Ok(_) => {}
                        Err(_) => return,
                    }
                }
                _ => {}
            },
            Err(_) => {
                let response = IpcResponse::Error("Invalid Message".to_string());
                match send_response_to_client(response, &mut stream).await {
                    Ok(_) => {}
                    Err(_) => return,
                }
            }
        }
    }
}

pub async fn start_ipc_server(download_manager: Arc<Mutex<DownloadManager>>) {
    let listener = match TcpListener::bind(IPC_SOCKET_ADDRESS).await {
        Ok(listener) => listener,
        Err(_) => {
            return;
        }
    };
    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                let download_manager_clone = download_manager.clone();
                tokio::task::spawn(handle_ipc_client(stream, download_manager_clone));
            }
            Err(_) => {}
        }
    }
}
