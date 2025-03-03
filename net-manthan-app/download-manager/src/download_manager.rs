use crossbeam_channel::{Receiver, Sender};
use download_engine::Download;
use download_engine::{
    config::NetManthanConfig,
    types::{IpcRequest, IpcResponse},
};

pub struct DownloadManager {
    pub config: NetManthanConfig,
    pub all_downloads: Vec<Download>,
}

impl DownloadManager {
    pub fn new(all_downloads: Vec<Download>, config: NetManthanConfig) -> Self {
        Self {
            config,
            all_downloads,
        }
    }

    pub fn start(
        &self,
        ipc_response_sender: Sender<IpcResponse>,
        ipc_request_receiver: Receiver<IpcRequest>,
    ) {
        loop {}
    }
}
