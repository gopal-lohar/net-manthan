use std::{fs, path::PathBuf};

use download_db_manager::connect_to_database;
use download_engine::config::NetManthanConfig;
use download_manager::DownloadManager;
use ipc_server::start_ipc_server;
use tracing::{debug, error};
use utils::logging;

pub mod download_db_manager;
mod download_manager;
mod ipc_server;

#[tokio::main]
async fn main() {
    // TODO: download failed if the downloads folder does not exist (create it or gracefully handle it)
    let config = match NetManthanConfig::load_config(PathBuf::from("./.dev/config.toml")) {
        Ok(config) => config,
        Err(e) => {
            error!("Failed to load config: {}", e);
            NetManthanConfig::get_default_config()
        }
    };

    if !&config.download_dir.exists() {
        match fs::create_dir_all(&config.download_dir) {
            Ok(_) => (),
            Err(e) => {
                error!("Failed to create download directory: {}", e);
                std::process::exit(1);
            }
        }
    }

    match logging::init_logger("Net Manthan", PathBuf::from(config.log_path.clone())) {
        Ok(_) => (),
        Err(e) => {
            error!("Failed to initialize logger: {}", e);
        }
    }

    let db_manager = match connect_to_database(&config.database_path) {
        Ok(db_manager) => {
            debug!("Initialized db_manager");
            db_manager
        }
        Err(e) => {
            error!("Failed to connect to database: {}", e);
            std::process::exit(1);
        }
    };

    let all_downloads = match db_manager.get_all_downloads() {
        Ok(downloads) => {
            debug!("Initialized all_downloads");
            downloads
        }
        Err(e) => {
            error!("Failed to get downloads: {}", e);
            std::process::exit(1);
        }
    };

    let ipc_server_address = config.get_ipc_server_address();
    // TODO: add an ipc secret or signing thing for security purposes (Supreme Leader Laughs)
    // we are two months away from enriching weapons grade uranium... to be used for IPC communication only
    let (ipc_response_sender, ipc_request_receiver) = match start_ipc_server(&ipc_server_address) {
        Ok((sender, receiver)) => {
            debug!("IPC server started at {}", ipc_server_address);
            (sender, receiver)
        }
        Err(err) => {
            error!("Failed to start IPC server: {}", err);
            std::process::exit(1);
        }
    };

    let mut i = 0;
    for download in &all_downloads {
        debug!("{}. Download: {}", i + 1, download.filename);
        for part in &download.parts {
            debug!(
                "Part: {}KB/{}KB",
                part.bytes_downloaded / 1024,
                part.total_bytes / 1024
            );
        }
        i += 1;
    }

    let mut download_manager = DownloadManager::new(db_manager, all_downloads, config);
    download_manager
        .start(ipc_response_sender, ipc_request_receiver)
        .await;
}
