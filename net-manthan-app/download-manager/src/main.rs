use std::path::Path;

use chrono::Utc;
use download_db_manager::DatabaseManager;

pub mod constants;
pub mod download_db_manager;
mod download_manager;
mod ipc_server;

#[tokio::main]
async fn main() {
    let db_path = Path::new("./.dev/downloads.db");
    if let Some(parent) = db_path.parent() {
        match std::fs::create_dir_all(parent) {
            Ok(_) => {}
            Err(e) => {
                eprintln!("Failed to create directory: {}", e);
                std::process::exit(1);
            }
        };
    }

    let mut db_manager = match DatabaseManager::new(db_path) {
        Ok(db_manager) => db_manager,
        Err(e) => {
            eprintln!("Failed to initialize database: {}", e);
            std::process::exit(1);
        }
    };

    let mut download = download_db_manager::Download {
        download_id: "".to_string(),
        filename: "example.txt".to_string(),
        path: "/path/to/download".to_string(),
        referrer: Some("http://example.com".to_string()),
        download_link: "http://example.com/download".to_string(),
        resumable: true,
        total_size: 1024,
        size_downloaded: 0,
        average_speed: 0,
        date_added: Utc::now(),
        date_finished: None,
        active_time: 0,
        parts: Vec::new(),
    };
    match db_manager.insert_download(&mut download) {
        Ok(_) => {
            println!("Succesfully inserted download")
        }
        Err(e) => {
            eprintln!("Failed to insert download: {}", e);
            std::process::exit(1);
        }
    }
}
