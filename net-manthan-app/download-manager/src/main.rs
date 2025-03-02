use std::path::PathBuf;

use download_db_manager::connect_to_database;
use download_manager::DownloadManager;
use ipc_server::start_ipc_server;
use net_manthan_core::config::NetManthanConfig;

pub mod download_db_manager;
mod download_manager;
mod ipc_server;

#[tokio::main]
async fn main() {
    let config = match NetManthanConfig::load_config(PathBuf::from("./.dev/config.toml")) {
        Ok(config) => config,
        Err(e) => NetManthanConfig::get_default_config(),
    };

    let db_manager = match connect_to_database(&config.database_path) {
        Ok(db_manager) => db_manager,
        Err(e) => {
            eprintln!("Failed to connect to database: {}", e);
            std::process::exit(1);
        }
    };

    let all_downloads = match db_manager.get_all_downloads() {
        Ok(downloads) => downloads,
        Err(e) => {
            eprintln!("Failed to get downloads: {}", e);
            std::process::exit(1);
        }
    };

    let ipc_server_address = format!("{}:{}", config.ipc_server_address, config.ipc_server_port);
    // TODO: add an ipc secret or signing thing for security purposes (Supreme Leader Laughs)
    // we are two months away from enriching weapons grade uranium... to be used for IPC communication only
    let (ipc_response_sender, ipc_request_receiver) = match start_ipc_server(&ipc_server_address) {
        Ok((sender, receiver)) => (sender, receiver),
        Err(_) => {
            // eprintln!("Failed to start IPC server: {}", e); // TODO: use logs
            std::process::exit(1);
        }
    };

    let download_manager = DownloadManager::new(all_downloads, config);
    download_manager.start(ipc_response_sender, ipc_request_receiver);
}
