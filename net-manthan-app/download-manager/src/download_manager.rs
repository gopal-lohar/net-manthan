use crossbeam_channel::{select, Receiver, Sender};
use download_engine::Download;
use download_engine::{
    config::NetManthanConfig,
    types::{IpcRequest, IpcResponse},
};
use tracing::{debug, error};

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


                    let response: IpcResponse = match msg{
                        Ok(ipc_request)=>{
                            match ipc_request {
                                IpcRequest::HeartBeat => IpcResponse::HeartBeat,
                                IpcRequest::ListDownloads{
                                    incomplete_only,
                                    detailed,
                                    limit
                                } =>{
                                    _ = incomplete_only;
                                    _ = detailed;
                                    _ = limit;
                                    let downloads = self.all_downloads.clone();
                                    IpcResponse::DownloadsList(downloads)
                                }
                                IpcRequest::GetConfig =>{
                                    let config = self.config.clone();
                                    IpcResponse::Config(config)
                                }
                                _ => IpcResponse::Error("Unsupported IPC request (for now)".to_string())
                            }

                        },
                        Err(e)=>{
                            error!("an error occurred while receiving IPC request: {}", e);
                            IpcResponse::Error(format!("IPC request error: {}", e))
                        }
                    };

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
