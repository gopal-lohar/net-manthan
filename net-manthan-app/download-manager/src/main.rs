use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

use download_db_manager::connect_to_database;
use download_manager::DownloadManager;
use net_manthan_core::config::NetManthanConfig;
use progress_manager::progress_manager;

pub mod constants;
pub mod download_db_manager;
mod download_manager;
mod ipc_server;
mod progress_manager;

#[tokio::main]
async fn main() {
    let config = match NetManthanConfig::load_config(PathBuf::from("./.dev/config.toml")) {
        Ok(config) => config,
        Err(e) => NetManthanConfig::get_default_config(),
    };

    let mut db_manager = match connect_to_database(&config.database_path) {
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

    let download_manager = Arc::new(Mutex::new(DownloadManager::new(all_downloads, config)));

    tokio::spawn(progress_manager(db_manager, download_manager.clone()));

    ipc_server::start_ipc_server(download_manager).await;
}
