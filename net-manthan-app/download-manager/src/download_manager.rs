use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use chrono::Utc;
use crossbeam_channel::{bounded, select, Receiver, Sender};
use download_engine::types::{DownloadRequest, DownloadStatus, PartProgress};
use download_engine::Download;
use download_engine::{
    config::NetManthanConfig,
    types::{IpcRequest, IpcResponse},
};
use tracing::{debug, error, info};

use crate::download_db_manager::DatabaseManager;

pub struct DownloadManager {
    pub db_manager: DatabaseManager,
    pub config: NetManthanConfig,
    pub all_downloads: Vec<Download>,
    pub aggregator_sender: Sender<Vec<PartProgress>>,
    pub aggregator_receiver: Receiver<Vec<PartProgress>>,
    pub active_download: HashMap<String, Arc<AtomicBool>>,
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
            active_download: HashMap::new(),
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
            IpcRequest::GetDownloads(status_vec) => {
                let downloads = self
                    .all_downloads
                    .clone()
                    .into_iter()
                    .filter(|d| {
                        status_vec.iter().any(|status| match (status, &d.status) {
                            (DownloadStatus::Completed(_), DownloadStatus::Completed(_)) => true,
                            (DownloadStatus::Failed(_), DownloadStatus::Failed(_)) => true,
                            (s1, s2) => s1 == s2,
                        })
                    })
                    .collect();
                IpcResponse::DownloadsList(downloads)
            }
            IpcRequest::GetActiveDownloads {} => {
                let active_ids: Vec<&String> = self.active_download.keys().collect();
                let active_downloads = self
                    .all_downloads
                    .iter()
                    .filter(|download| active_ids.contains(&&download.download_id))
                    .cloned()
                    .collect();
                IpcResponse::DownloadsList(active_downloads)
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
                info!("Starting New Download");
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
                        let cancel_handle = Arc::new(AtomicBool::new(false));
                        download
                            .start(
                                self.aggregator_sender.clone(),
                                self.config.clone(),
                                cancel_handle.clone(),
                            )
                            .await;
                        info!("New Download started");
                        self.all_downloads.push(download.clone());
                        match self.db_manager.insert_download(&mut download) {
                            Ok(_) => {
                                self.active_download
                                    .insert(download.download_id, cancel_handle);
                                IpcResponse::Success {}
                            }
                            Err(err) => IpcResponse::Error(err.to_string()),
                        }
                    }
                    Err(err) => IpcResponse::Error(err.to_string()),
                };
                download
            }
            IpcRequest::ChangeDownloadStatus {
                download_id,
                download_status,
            } => {
                if let Some(download) = self
                    .all_downloads
                    .iter_mut()
                    .find(|d| d.download_id == download_id)
                {
                    match download_status {
                        DownloadStatus::Paused => {
                            // TODO: some checks are needed
                            if let Some(cancel_token) = self.active_download.remove(&download_id) {
                                cancel_token.store(true, Ordering::Relaxed);
                                download.status = DownloadStatus::Paused;
                                info!("Download {} paused", download_id);
                                IpcResponse::Success {}
                            } else {
                                IpcResponse::Error("Download not active".to_string())
                            }
                        }
                        DownloadStatus::Downloading => {
                            let cancel_handle = Arc::new(AtomicBool::new(false));
                            info!("resuming the download");
                            download
                                .start(
                                    self.aggregator_sender.clone(),
                                    self.config.clone(),
                                    cancel_handle.clone(),
                                )
                                .await;
                            self.active_download
                                .insert(download.download_id.clone(), cancel_handle);
                            info!("Download started");
                            IpcResponse::Success {}
                        }
                        _ => IpcResponse::Error("Unsupported download status".to_string()),
                    }
                } else {
                    IpcResponse::Error("Download not found".to_string())
                }
            }
            IpcRequest::GetConfig => {
                let config = self.config.clone();
                IpcResponse::Config(config)
            } // _ => IpcResponse::Error("Unsupported IPC request (for now)".to_string()),
        }
    }

    pub fn handle_progress_update(&mut self, progress_vec: Vec<PartProgress>) {
        // TODO: close download threads after it's done
        // TODO: the threads (neither download nor aggregate) are not really handled properly
        if let Some(download_index) = self.all_downloads.iter().position(|download| {
            download
                .parts
                .iter()
                .position(|part| part.part_id == progress_vec[0].part_id)
                .is_some()
        }) {
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

            if progress_vec
                .iter()
                .all(|p| matches!(p.status, DownloadStatus::Completed(_)))
            {
                download.status = DownloadStatus::Completed(
                    if let Some(time) = progress_vec
                        .iter()
                        .filter_map(|item| {
                            if let DownloadStatus::Completed(time) = item.status {
                                Some(time)
                            } else {
                                None
                            }
                        })
                        .max()
                    {
                        time
                    } else {
                        Utc::now()
                    },
                );
            }
        } else {
            error!("download not found");
        }
    }
}
