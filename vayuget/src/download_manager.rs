use std::{collections::VecDeque, time::Duration};

use engine::{
    download::start_download::start_download,
    types::{
        chunks::ChunksInfo, download::Download, download_handle::DownloadHandle,
        request::DownloadRequest,
    },
};
use tokio::{
    sync::{
        mpsc::{self, Sender},
        oneshot,
    },
    time::interval,
};
use tracing::{debug, info, trace};
use utils::{
    pretty_print_downloads::pretty_print_downloads,
    rpc::{
        NativeRpcSettings, RpcConfig,
        messages::{RpcRequest, RpcResponse},
        server::{ManagerCommand, RpcServer, RpcServerHandle},
    },
};

pub struct DownloadManager {
    all: Vec<Download>,
    // active, waiting_info and queued are part of active download
    active: Vec<DownloadHandle>,
    /// downloads that need to start but don't have info.
    /// As soon as info arrives, it will either be queued or get active
    waiting_info: Vec<i64>,
    queued: VecDeque<i64>,
    /// List of dodwnload id's that probably need to be updated in db (i.e. were self.all was changed)
    dirty: Vec<i64>,
    rpc_server: Option<RpcServer>,
    load_info_tx: Option<Sender<(i64, DownloadRequest)>>,
    max_active_downloads: usize,
    pretty_print: bool,
}

impl DownloadManager {
    pub async fn new(max_active_downloads: usize, pretty_print: bool) -> Self {
        Self {
            all: Vec::new(),
            active: Vec::new(),
            waiting_info: Vec::new(),
            queued: VecDeque::new(),
            dirty: Vec::new(),
            rpc_server: None,
            load_info_tx: None,
            max_active_downloads,
            pretty_print,
        }
    }

    /// Starts the RPC server in the same thread.
    pub async fn start_server(&mut self, secret: String, sender: mpsc::Sender<ManagerCommand>) {
        let rpc_server_handle = RpcServerHandle {
            command_sender: sender,
            secret,
        };
        let mut rpc_server = RpcServer::new(
            &RpcConfig {
                native_rpc_settings: NativeRpcSettings {
                    address: "/tmp/vayu.sock".into(),
                    allow_all_users: true,
                },
            },
            rpc_server_handle,
        );
        rpc_server.start().await;
        self.rpc_server = Some(rpc_server);
    }

    /// [ ] Updates the progress of each active download
    ///
    /// [ ] Updates handles the download queue and removes the puased/failed download from active downloads
    ///
    /// [ ] Write all that in the database
    async fn update(&mut self) {
        for active_d in &mut self.active {
            active_d.update_progress().await;
            self.dirty.push(active_d.id);
        }
        if self.pretty_print {
            pretty_print_downloads(&self.active.iter().map(|d| d.core.clone()).collect(), true);
        }
    }

    pub async fn run(
        mut self,
        mut receiver: mpsc::Receiver<ManagerCommand>,
        command_sender: mpsc::Sender<ManagerCommand>,
        mut shutdown_rx: oneshot::Receiver<()>,
    ) {
        let (load_info_tx, rx) = mpsc::channel::<(i64, DownloadRequest)>(10);
        self.load_info_tx = Some(load_info_tx);
        tokio::task::spawn(Self::load_info_routine(command_sender, rx));
        let mut interval = interval(Duration::from_millis(250));
        loop {
            // Biased selection ensures the interval is checked first
            tokio::select! {
                biased; // <-- Prioritize branches in order
                // Check interval first to avoid starvation
                _ = interval.tick() => {
                    self.update().await;
                }

                _ = &mut shutdown_rx => {
                    break;
                }

                // Process commands only if interval is not ready
                cmd = receiver.recv() => {
                    if let Some(cmd) = cmd {
                        let shutdown = matches!(cmd.request, RpcRequest::Shutdown);
                        self.handle_command(cmd).await;
                        if shutdown{
                            break
                        }
                    } else {
                        continue;
                    }
                }
            }
        }
        self.shutdown().await;
        self.update().await;
    }

    /// should only be called if the info has loaded
    /// if slot available, starts the download and delete it from the queue
    /// else add it to the queue
    async fn start_download(&mut self, download: Download) {
        let d_id = download.id;
        if self.active.len() < self.max_active_downloads {
            if let Some(handle) = start_download(download).await {
                self.queued.retain(|d| *d != d_id);
                self.active.insert(0, handle);
            } // don't need to handle else because info has loaded
        } else {
            trace!(
                "reached maximum limit for active downloads, pushing {} to queue",
                d_id
            );
            if self.queued.iter().find(|d| **d == d_id).is_none() {
                self.queued.push_back(d_id);
            }
        }
    }

    async fn download(&mut self, mut download: Download) {
        self.dirty.push(download.id);
        if download.has_info_loaded() {
            self.start_download(download.clone()).await;
        } else {
            if let Some(tx) = &self.load_info_tx {
                trace!(
                    "Info does not exist, requesting info for url: {}",
                    download.request.url
                );
                match tx.send((download.id, download.request.clone())).await {
                    Ok(_) => {
                        self.waiting_info.push(download.id);
                    }
                    Err(err) => {
                        trace!("Failed to load download info: {}", err.to_string());
                        download.chunks_info = ChunksInfo::ErrorLoadingInfo(format!(
                            "Failed to load download info: {}",
                            err.to_string()
                        ))
                    }
                }
            }
        }
        self.all.insert(0, download.clone());
    }

    async fn load_info_routine(
        command_sender: mpsc::Sender<ManagerCommand>,
        mut rx: mpsc::Receiver<(i64, DownloadRequest)>,
    ) {
        loop {
            if let Some((id, request)) = rx.recv().await {
                info!("Loadig download info for id: {}, link: {}", id, request.url);
                let _ = ManagerCommand::send(
                    RpcRequest::SetDownloadInfo((
                        id,
                        match request.load_download_info().await {
                            Ok(info) => Ok(info),
                            Err(err) => Err(format!("Failed to load download info: {}", err)),
                        },
                    )),
                    &command_sender,
                )
                .await;
            } else {
            };
        }
    }

    async fn handle_command(&mut self, command: ManagerCommand) {
        let respond_to = command.respond_to;
        let response = match command.request {
            RpcRequest::Heartbeat => RpcResponse::Heartbeat,
            RpcRequest::DownloadRequest(request) => {
                debug!("recieved download request for url: {}", request.request.url);
                let download = Download::new(request).await;
                self.download(download).await;
                RpcResponse::Recieved
            }
            RpcRequest::SetDownloadInfo((id, maybe_info)) => {
                if let Some(download) = self.all.iter_mut().find(|d| d.id == id) {
                    match maybe_info {
                        Ok(info) => {
                            download.set_info(info).await;
                        }
                        Err(err) => {
                            download.chunks_info = ChunksInfo::ErrorLoadingInfo(err.to_string());
                        }
                    }
                    self.dirty.push(id);
                }

                if self.waiting_info.iter().find(|w| **w == id).is_some() {
                    let start_download = self.all.iter().find(|d| d.id == id);
                    if let Some(download) = start_download {
                        if download.has_info_loaded() {
                            self.start_download(download.clone()).await;
                            self.waiting_info.retain(|d| *d != id);
                        }
                    }
                }

                RpcResponse::Recieved
            }
            RpcRequest::Shutdown => {
                // shutdown handle in caller
                RpcResponse::Recieved
            }
            _ => RpcResponse::RequestNotSupported,
        };

        let _ = respond_to.send(response);
    }

    async fn shutdown(&mut self) {
        if self.pretty_print {
            pretty_print_downloads(&self.active.iter().map(|d| d.core.clone()).collect(), false);
        }
        info!("Shutting down the Download Manager...");
        if let Some(handle) = self.rpc_server.take() {
            debug!("Shutting down the RPC server...");
            handle.shutdown().await;
        }
    }
}
