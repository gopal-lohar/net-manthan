use chrono::{DateTime, Utc};
use engine::{
    download::start_download::start_download,
    types::{
        chunks::ChunksInfo, download::Download, download_handle::DownloadHandle,
        request::DownloadRequest, status::DownloadStatus,
    },
};
use std::{
    collections::{HashMap, VecDeque},
    time::Duration,
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
    queue: VecDeque<i64>,
    /// List of dodwnload id's that probably need to be updated in db (i.e. were self.all was changed)
    ///
    /// Can be converted into a Hashmap
    dirty: Vec<i64>,
    rpc_server: Option<RpcServer>,
    load_info_tx: Option<Sender<(i64, DownloadRequest)>>,
    command_sender: Sender<ManagerCommand>,
    last_updated: DateTime<Utc>,
    max_active_downloads: usize,
    shutting_down: bool,
    daemon_mode: bool,
    pretty_print: bool,
}

impl DownloadManager {
    pub async fn new(
        max_active_downloads: usize,
        command_sender: mpsc::Sender<ManagerCommand>,
        daemon_mode: bool,
        pretty_print: bool,
    ) -> Self {
        Self {
            all: Vec::new(),
            active: Vec::new(),
            waiting_info: Vec::new(),
            queue: VecDeque::new(),
            dirty: Vec::new(),
            rpc_server: None,
            load_info_tx: None,
            command_sender,
            last_updated: Utc::now(),
            max_active_downloads,
            shutting_down: false,
            daemon_mode,
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
        // we need to do a lot of things with all
        let mut id_to_index: HashMap<i64, usize> = HashMap::new();
        for (index, download) in self.all.iter().enumerate() {
            id_to_index.insert(download.id, index);
        }

        let now = Utc::now();
        let delta = now - self.last_updated;
        for handle in &mut self.active {
            handle.time_stamps.date_updated = now;
            handle.time_stamps.active_time += delta;
            if matches!(handle.get_status(), DownloadStatus::Complete) {
                handle.time_stamps.date_completed = Some(now);
            }
            handle.update_progress().await;
            if let Some(&index) = id_to_index.get(&handle.core.id) {
                self.all[index] = handle.core.clone();
                if !self.dirty.contains(&handle.core.id) {
                    self.dirty.push(handle.core.id);
                }
            }
        }
        self.last_updated = now;

        if !self.shutting_down && self.pretty_print {
            pretty_print_downloads(&self.all.iter().map(|d| d.clone()).collect(), true);
        }

        self.active.retain(|handle| {
            matches!(
                handle.get_status(),
                DownloadStatus::Connecting | DownloadStatus::Retrying | DownloadStatus::Downloading
            )
        });

        // Handle the queue
        if self.active.len() < self.max_active_downloads && self.queue.len() > 0 {
            if let Some(next) = self.queue.pop_front() {
                let download = self.all.iter().find(|d| d.id == next);
                if let Some(download) = download {
                    self.download(download.clone()).await;
                }
            }
        }

        // in non daemon mode, if no downloads are active and none are in a position to start, just shut down
        if !self.daemon_mode
            && !self.shutting_down
            && !self.all.iter().any(|d| match &d.chunks_info {
                ChunksInfo::InfoNotLoaded => true,
                ChunksInfo::Queued => true,
                ChunksInfo::InfoLoaded(_) => {
                    matches!(
                        d.get_status(),
                        DownloadStatus::Created
                            | DownloadStatus::Queued
                            | DownloadStatus::Connecting
                            | DownloadStatus::Retrying
                            | DownloadStatus::Downloading
                    )
                }
                _ => false,
            })
        {
            info!("No downloads are currently active. Shutting down...");
            ManagerCommand::fire_forget(RpcRequest::Shutdown, &self.command_sender).await;
        }

        // TODO: write dirty downloads
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
                            info!("DownloadManager shutting down");
                            break
                        }
                    } else {
                        // All senders are dropped, but there may be downloads in progress
                        // Although there is a sender that won't be dropped, the ctrl_c sender, but we still want to continue, just in case
                        continue;
                    }
                }
            }
        }
        self.shutdown().await;
        self.update().await;
    }

    fn update_original_download(&mut self, download: Download) {
        let id = download.id;
        if let Some(original) = self.all.iter_mut().find(|d| d.id == id) {
            *original = download;
            if !self.dirty.contains(&id) {
                self.dirty.push(id);
            }
        }
    }

    /// should only be called if the info has loaded
    /// if slot available, starts the download and delete it from the queue
    /// else add it to the queue
    async fn start_download(&mut self, download: Download) {
        let d_id = download.id;
        if self.active.len() < self.max_active_downloads {
            if let Some(handle) = start_download(download).await {
                self.queue.retain(|d| *d != d_id);
                self.active.insert(0, handle);
                self.active[0].update_progress().await;
                self.update_original_download(self.active[0].core.clone());
            } // don't need to handle else because info has loaded
        } else {
            trace!(
                "reached maximum limit for active downloads, pushing {} to queue",
                d_id
            );
            if self.queue.iter().find(|d| **d == d_id).is_none() {
                self.queue.push_back(d_id);
                if let Some(d) = self.all.iter_mut().find(|d| d.id == d_id) {
                    d.set_status(DownloadStatus::Queued);
                };
            }
        }
    }

    /// start a new download (fetches download info if not already loaded)
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

        if self.all.iter().find(|d| d.id == download.id).is_none() {
            self.all.insert(0, download.clone());
        }
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
                            if !download.has_info_loaded() {
                                download.set_info(info).await;
                            }
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
                            // not calling self.download here because that will cause infinite loop
                            self.start_download(download.clone()).await;
                            self.waiting_info.retain(|d| *d != id);
                        }
                    }
                }

                RpcResponse::Recieved
            }
            RpcRequest::GetDownloads(_) => {
                let downloads = self.all.iter().map(|d| d.clone()).collect();
                RpcResponse::Downloads(downloads)
            }
            RpcRequest::Shutdown => {
                // shutdown handle in caller
                RpcResponse::Recieved
            }
            RpcRequest::PauseDownload(id) => {
                debug!("Reqest for pause {id}");
                if let Some(index) = self.active.iter().position(|d| d.id == id) {
                    self.active[index].pause().await;
                    RpcResponse::Success
                } else {
                    RpcResponse::Error("Download not found".into())
                }
            }
            RpcRequest::ResumeDownload(id) => {
                debug!("Reqest for resume {id}");
                if let Some(download) = self.all.iter().find(|d| d.id == id) {
                    self.download(download.clone()).await;
                    RpcResponse::Success
                } else {
                    RpcResponse::Error("Download not found".into())
                }
            }
            _ => RpcResponse::RequestNotSupported,
        };

        let _ = respond_to.send(response);
    }

    async fn shutdown(&mut self) {
        self.shutting_down = true;
        if self.pretty_print {
            pretty_print_downloads(&self.all.iter().map(|d| d.clone()).collect(), false);
        }
        info!("Shutting down the Download Manager...");
        if let Some(handle) = self.rpc_server.take() {
            debug!("Shutting down the RPC server...");
            handle.shutdown().await;
        }
    }
}
