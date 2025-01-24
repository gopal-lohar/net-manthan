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
