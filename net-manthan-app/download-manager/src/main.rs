pub mod download_db_manager;

use download_db_manager::DatabaseManager;
use net_manthan_core::{download, DownloadRequest};
use std::{
    io::{Read, Write},
    os::unix::net::UnixListener,
    path::Path,
};
use tracing::{error, info, Level};

const SOCKET_PATH: &str = "/tmp/net-manthan.sock";

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    let db_path = Path::new("/tmp/downloads.db");
    let db_manager = DatabaseManager::new(db_path);

    if Path::new(SOCKET_PATH).exists() {
        std::fs::remove_file(SOCKET_PATH).unwrap();
    }

    let listener = UnixListener::bind(SOCKET_PATH).unwrap();
    info!("Service started on {}", SOCKET_PATH);

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let mut buffer = Vec::new();
                match stream.read_to_end(&mut buffer) {
                    Ok(_) => match serde_json::from_slice::<DownloadRequest>(&buffer) {
                        Ok(request) => {
                            info!("Download request received");
                            info!("Starting download");

                            // pub struct Download {
                            //     pub id: Option<i64>,
                            //     pub url: String,
                            //     pub filename: String,
                            //     pub mime_type: Option<String>,
                            //     pub total_size: u64,
                            //     pub status: DownloadStatus,
                            // }
                            match &db_manager {
                                Ok(db_manager) => {
                                    match db_manager.insert_download(
                                        &download_db_manager::Download {
                                            id: None,
                                            url: request.url.clone(),
                                            filename: request.filename.clone(),
                                            mime_type: None,
                                            total_size: 0,
                                            status: download_db_manager::DownloadStatus::Pending,
                                        },
                                    ) {
                                        Ok(_) => {
                                            info!("Download inserted into database");
                                        }
                                        Err(e) => {
                                            error!(
                                                "Failed to insert download into database: {}",
                                                e
                                            );
                                        }
                                    }

                                    match db_manager.get_all_downloads() {
                                        Ok(downloads) => {
                                            info!("All downloads: {:?}", downloads);
                                        }
                                        Err(e) => {
                                            error!("Failed to get all downloads: {}", e);
                                        }
                                    }
                                }
                                Err(e) => {
                                    error!("Failed to create database manager: {}", e);
                                }
                            }

                            match download(request).await {
                                Ok(_) => {
                                    info!("Download finished");
                                }
                                Err(e) => {
                                    error!("Download Failed: {}", e);
                                }
                            }

                            let _ = stream.write_all(b"OK");
                        }
                        Err(e) => error!("Failed to parse request: {}", e),
                    },
                    Err(e) => error!("Failed to read from socket: {}", e),
                }
            }
            Err(e) => error!("Connection failed: {}", e),
        }
    }
}
