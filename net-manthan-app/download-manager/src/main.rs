use std::{
    path::Path,
    sync::{Arc, Mutex},
};

use download_db_manager::connect_to_database;
use download_manager::DownloadManager;

pub mod constants;
pub mod download_db_manager;
mod download_manager;
mod ipc_server;

#[tokio::main]
async fn main() {
    let db_path = Path::new("./.dev/downloads.db");
    let mut db_manager = match connect_to_database(db_path) {
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

    let download_manager = Arc::new(Mutex::new(DownloadManager::new(all_downloads)));
    ipc_server::start_ipc_server(download_manager).await;
}
