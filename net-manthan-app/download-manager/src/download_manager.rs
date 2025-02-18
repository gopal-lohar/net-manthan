use std::sync::{atomic::AtomicBool, Arc};

use crate::download_db_manager::Download;
use net_manthan_core::types::{DownloadRequest, PartProgress};
use net_manthan_core::types::{IpcRequest, IpcResponse};
use net_manthan_core::{config::NetManthanConfig, download};
use tokio::sync::broadcast;

pub struct DownloadManager {
    pub config: NetManthanConfig,
    pub all_downloads: Vec<Download>,
}

pub struct DownloadHandle {
    pub cancel_token: Arc<AtomicBool>,
    pub progress_receiver: broadcast::Receiver<Vec<PartProgress>>,
}

impl DownloadManager {
    pub fn new(all_downloads: Vec<Download>, config: NetManthanConfig) -> Self {
        Self {
            config,
            all_downloads,
        }
    }

    pub fn handle_ipc_request(&mut self, request: IpcRequest) -> IpcResponse {
        match request {
            IpcRequest::HeartBeat => IpcResponse::HeartBeat,
            IpcRequest::StartDownload {
                url,
                output_path,
                thread_count,
                headers,
            } => {
                // download_manager.lock().unwrap().start_download(
                //     download_id,
                //     DownloadRequest {
                //         url,
                //         filepath: output_path.unwrap_or("/tmp/test".into()),
                //         headers,
                //         parts: None,
                //         config: DownloadRequestConfig {
                //             thread_count: thread_count.unwrap_or(5),
                //             buffer_size: 1024 * 1024,
                //             update_interval: Duration::seconds(1),
                //         },
                //     },
                // );
                println!("Download started");
                IpcResponse::Success
            }
            _ => IpcResponse::Error("Invalid request".into()),
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
    }
}
