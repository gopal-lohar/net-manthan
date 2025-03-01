use std::sync::{Arc, Mutex};

use crate::{download_db_manager::DatabaseManager, download_manager::DownloadManager};

pub async fn progress_manager(
    mut db_manager: DatabaseManager,
    download_manager: Arc<Mutex<DownloadManager>>,
) {
    println!("Started");
}
