use crossbeam_channel::{bounded, select, Receiver, Sender};
use download_engine::types::{DownloadRequest, PartProgress};
use download_engine::Download;
use download_engine::{
    config::NetManthanConfig,
    types::{IpcRequest, IpcResponse},
};
use tracing::{debug, error};

use crate::download_db_manager::DatabaseManager;

pub struct DownloadManager {
    pub db_manager: DatabaseManager,
    pub config: NetManthanConfig,
    pub all_downloads: Vec<Download>,
    pub aggregator_sender: Sender<Vec<PartProgress>>,
    pub aggregator_receiver: Receiver<Vec<PartProgress>>,
}

impl DownloadManager {
    pub fn new(
        db_manager: DatabaseManager,
        all_downloads: Vec<Download>,
        config: NetManthanConfig,
    ) -> Self {
        let (aggregator_sender, aggregator_receiver) = bounded::<Vec<PartProgress>>(100);
        Self {
            db_manager,
            config,
            all_downloads,
            aggregator_sender,
            aggregator_receiver,
        }
    }

    pub async fn start(
        &mut self,
        ipc_response_sender: Sender<IpcResponse>,
        ipc_request_receiver: Receiver<IpcRequest>,
    ) {
        loop {
            select! {
                recv(ipc_request_receiver) -> msg =>{
                    debug!("Received IPC message {:?}", msg);
                    let response: IpcResponse = match msg{
                        Ok(ipc_request)=>{
                            self.handle_ipc_request(ipc_request).await
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
                },
                recv(self.aggregator_receiver) -> msg =>{
                    match msg{
                        Ok(progress_vec) => {
                            self.handle_progress_update(progress_vec);
                        },
                        Err(e) => {
                            error!("an error occurred while receiving progress update: {}", e);
                        }
                    };
                }
            }
        }
    }

    pub async fn handle_ipc_request(&mut self, request: IpcRequest) -> IpcResponse {
        match request {
            IpcRequest::HeartBeat => IpcResponse::HeartBeat,
            IpcRequest::ListDownloads {
                incomplete_only,
                detailed,
                limit,
            } => {
                _ = incomplete_only;
                _ = detailed;
                _ = limit;
                let downloads = self.all_downloads.clone();
                IpcResponse::DownloadsList(downloads)
            }
            IpcRequest::StartDownload {
                url,
                output_path,
                thread_count,
                headers,
            } => {
                _ = output_path;
                _ = thread_count;
                _ = headers;
                let download = match Download::new(
                    DownloadRequest {
                        url,
                        referrer: None,
                        filepath: None,
                        filename: None,
                        headers: None,
                    },
                    self.config.clone(),
                )
                .await
                {
                    Ok(mut download) => {
                        let cancel_handle =
                            download.start(self.aggregator_sender.clone(), self.config.clone());
                        self.all_downloads.push(download.clone());
                        match self.db_manager.insert_download(&mut download) {
                            Ok(id) => IpcResponse::Success {},
                            Err(err) => IpcResponse::Error(err.to_string()),
                        }
                    }
                    Err(err) => IpcResponse::Error(err.to_string()),
                };
                download
            }
            IpcRequest::GetConfig => {
                let config = self.config.clone();
                IpcResponse::Config(config)
            }
            _ => IpcResponse::Error("Unsupported IPC request (for now)".to_string()),
        }
    }

    pub fn handle_progress_update(&mut self, progress_vec: Vec<PartProgress>) {
        if let Some(download_index) = self
            .all_downloads
            .iter()
            .position(|download| download.download_id == progress_vec[0].download_id)
        {
            let download = &mut self.all_downloads[download_index];

            for download_part in &mut download.parts {
                for progress in &progress_vec {
                    if progress.part_id == download_part.part_id {
                        download_part.bytes_downloaded = progress.bytes_downloaded;

                        if let Err(e) = self
                            .db_manager
                            .update_part_progress(&download_part.part_id, progress.bytes_downloaded)
                        {
                            error!("failed to update download: {}", e);
                        }
                    }
                }
            }
        } else {
            error!("download not found");
        }
    }
}
