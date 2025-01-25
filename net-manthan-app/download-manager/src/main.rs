pub mod download_db_manager;

use bincode;
use std::sync::Arc;
// use download_db_manager::DatabaseManager;
use net_manthan_core::download;
// use serde::{Deserialize, Serialize};
// use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::{
    io::{Read, Write},
    // path::Path,
};
use tokio::runtime::Runtime;
use tracing::{error, info, Level};
use utils::Message;
use utils::IPC_SOCKET_ADDRESS;

// const SOCKET_PATH: &str = "/tmp/net-manthan.sock";

async fn handle_ipc_client(mut stream: TcpStream) {
    info!("New connection: {}", stream.peer_addr().unwrap());
    let mut buffer = Vec::new();

    loop {
        buffer.clear();
        // Read the message length (8 bytes for u64)
        let mut len_bytes = [0u8; 8];
        if stream.read_exact(&mut len_bytes).is_err() {
            info!("Client disconnected");
            return;
        }
        let msg_len = u64::from_le_bytes(len_bytes) as usize;

        // Read the actual message
        buffer.resize(msg_len, 0);
        if stream.read_exact(&mut buffer).is_err() {
            info!("Client disconnected");
            return;
        }

        match bincode::deserialize::<Message>(&buffer) {
            Ok(message) => {
                info!("Received: {:?}", message);
                let response = match message {
                    Message::HeartBeat => Message::HeartBeat,
                    Message::DownloadRequest(request) => {
                        match download(request).await {
                            Ok(_) => {
                                info!("Download finished");
                            }
                            Err(e) => {
                                error!("Download Failed: {}", e);
                            }
                        }
                        Message::DownnloadResponse("Download Finished".to_string())
                    }
                    _ => Message::InvalidMessage,
                };
                let serialized = bincode::serialize(&response).unwrap();
                if let Err(e) = stream.write_all(&(serialized.len() as u64).to_le_bytes()) {
                    error!("Failed to send response length: {}", e);
                    return;
                }
                if let Err(e) = stream.write_all(&serialized) {
                    error!("Failed to send response: {}", e);
                    return;
                }
                stream
                    .flush()
                    .unwrap_or_else(|e| error!("Failed to flush: {}", e));
            }
            Err(e) => {
                error!("Deserialization error: {}", e);
                return;
            }
        }
    }
}

fn main() {
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();
    let rt = Arc::new(Runtime::new().unwrap());

    // let db_path = Path::new("/tmp/downloads.db");
    // let db_manager = DatabaseManager::new(db_path);

    let listener = match TcpListener::bind(IPC_SOCKET_ADDRESS) {
        Ok(listener) => listener,
        Err(e) => {
            error!("Failed to bind to {}: {}", IPC_SOCKET_ADDRESS, e);
            return;
        }
    };

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let rt_clone = rt.clone();
                thread::spawn(move || {
                    rt_clone.block_on(async {
                        handle_ipc_client(stream).await;
                    });
                });
            }
            Err(e) => error!("Connection failed: {}", e),
        }
    }

    // if Path::new(SOCKET_PATH).exists() {
    //     std::fs::remove_file(SOCKET_PATH).unwrap();
    // }

    // let listener = UnixListener::bind(SOCKET_PATH).unwrap();
    // info!("Service started on {}", SOCKET_PATH);

    // for stream in listener.incoming() {
    //     match stream {
    //         Ok(mut stream) => {
    //             let mut buffer = Vec::new();
    //             match stream.read_to_end(&mut buffer) {
    //                 Ok(_) => match serde_json::from_slice::<DownloadRequest>(&buffer) {
    //                     Ok(request) => {
    //                         info!("Download request received");
    //                         info!("Starting download");

    //                         // pub struct Download {
    //                         //     pub id: Option<i64>,
    //                         //     pub url: String,
    //                         //     pub filename: String,
    //                         //     pub mime_type: Option<String>,
    //                         //     pub total_size: u64,
    //                         //     pub status: DownloadStatus,
    //                         // }
    //                         match &db_manager {
    //                             Ok(db_manager) => {
    //                                 match db_manager.insert_download(
    //                                     &download_db_manager::Download {
    //                                         id: None,
    //                                         url: request.url.clone(),
    //                                         filename: request.filename.clone(),
    //                                         mime_type: None,
    //                                         total_size: 0,
    //                                         status: download_db_manager::DownloadStatus::Pending,
    //                                     },
    //                                 ) {
    //                                     Ok(_) => {
    //                                         info!("Download inserted into database");
    //                                     }
    //                                     Err(e) => {
    //                                         error!(
    //                                             "Failed to insert download into database: {}",
    //                                             e
    //                                         );
    //                                     }
    //                                 }

    //                                 match db_manager.get_all_downloads() {
    //                                     Ok(downloads) => {
    //                                         info!("All downloads: {:?}", downloads);
    //                                     }
    //                                     Err(e) => {
    //                                         error!("Failed to get all downloads: {}", e);
    //                                     }
    //                                 }
    //                             }
    //                             Err(e) => {
    //                                 error!("Failed to create database manager: {}", e);
    //                             }
    //                         }

    //                         match download(request).await {
    //                             Ok(_) => {
    //                                 info!("Download finished");
    //                             }
    //                             Err(e) => {
    //                                 error!("Download Failed: {}", e);
    //                             }
    //                         }

    //                         let _ = stream.write_all(b"OK");
    //                     }
    //                     Err(e) => error!("Failed to parse request: {}", e),
    //                 },
    //                 Err(e) => error!("Failed to read from socket: {}", e),
    //             }
    //         }
    //         Err(e) => error!("Connection failed: {}", e),
    //     }
    // }
}
