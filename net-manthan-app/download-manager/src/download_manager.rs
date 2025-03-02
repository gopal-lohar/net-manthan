use crate::download_db_manager::Download;
use crossbeam_channel::{Receiver, Sender};
use net_manthan_core::{
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
    }
}
