use std::sync::{Arc, Mutex};

use download_manager::DownloadManager;

pub mod constants;
pub mod download_db_manager;
mod download_manager;
mod ipc_server;

#[tokio::main]
async fn main() {
    let download_manager = Arc::new(Mutex::new(DownloadManager::new()));
    ipc_server::start_ipc_server(download_manager).await;
}
