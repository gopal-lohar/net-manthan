use crossbeam_channel::{select, Receiver, Sender};
use download_engine::Download;
use download_engine::{
    config::NetManthanConfig,
    types::{IpcRequest, IpcResponse},
};
use tracing::debug;

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
        loop {
            select! {
                recv(ipc_request_receiver) -> msg =>{
                    debug!("Received IPC message {:?}", msg);
                    let response = IpcResponse::Success;
                    match ipc_response_sender.send(response.clone()){
                        Ok(_)=> {
                            debug!("sent IPC response message {:?}", response);
                        },
                        Err(_)=>{

                        }
                    };
                }
            }
        }
    }
}
