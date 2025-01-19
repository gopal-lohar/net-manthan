mod download;

use std::{
    io::{Read, Write},
    os::unix::net::UnixListener,
    path::Path,
};
use tracing::{error, info, Level};

const SOCKET_PATH: &str = "/tmp/net-manthan.sock";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
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
                    Ok(_) => match serde_json::from_slice::<download::DownloadRequest>(&buffer) {
                        Ok(request) => {
                            info!("Download request received: {:?}", request);
                            info!("Starting download");

                            download::handle_download(request).await?;

                            info!("Download finished");

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

    Ok(())
}
