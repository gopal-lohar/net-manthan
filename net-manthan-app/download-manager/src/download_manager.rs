use std::{
    collections::HashMap,
    sync::{atomic::AtomicBool, Arc},
};

use net_manthan_core::download;
use net_manthan_core::types::{DownloadRequest, PartProgress};
use tokio::sync::broadcast;

pub struct DownloadHandle {
    pub cancel_token: Arc<AtomicBool>,
    pub progress_receiver: broadcast::Receiver<Vec<PartProgress>>,
}

pub struct DownloadManager {
    pub active_downloads: HashMap<u64, DownloadHandle>,
}

impl DownloadManager {
    pub fn new() -> Self {
        Self {
            active_downloads: HashMap::new(),
        }
    }

    pub fn start_download(&mut self, download_id: u64, request: DownloadRequest) {
        let (broadcast_sender, broadcast_receiver) = broadcast::channel(100);
        let cancel_token = Arc::new(AtomicBool::new(false));

        let handle = DownloadHandle {
            cancel_token,
            progress_receiver: broadcast_receiver.resubscribe(),
        };

        tokio::spawn(download(
            request,
            handle.cancel_token.clone(),
            broadcast_sender,
        ));

        self.active_downloads.insert(download_id, handle);
    }
}
