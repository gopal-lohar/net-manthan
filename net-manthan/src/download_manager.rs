use download_engine::{Download, download_config::DownloadConfig};
use tokio::{
    sync::mpsc,
    time::{Duration, interval},
};
use utils::{
    conversion::{convert_to_download_proto, convert_to_download_req},
    rpc::ipc_server::{ManagerCommand, RpcServerHandle as DownloadManagerHandle},
    rpc_types::{
        DownloadList, Error as ErrorProto, GetDownload, RpcResponse, rpc_request::Request,
        rpc_response::Response,
    },
};

pub struct DownloadManager {
    all_downloads: Vec<Download>,
}

impl DownloadManager {
    pub fn new() -> DownloadManagerHandle {
        let (sender, receiver) = mpsc::channel(10);
        let handle = DownloadManagerHandle {
            command_sender: sender,
        };

        // Create and start the manager in its own thread
        let manager = Self {
            all_downloads: Vec::new(),
        };
        tokio::spawn(manager.run(receiver));

        handle
    }

    async fn run(mut self, mut receiver: mpsc::Receiver<ManagerCommand>) {
        let mut interval = interval(Duration::from_millis(250));
        loop {
            // Biased selection ensures the interval is checked first
            tokio::select! {
                biased; // <-- Prioritize branches in order
                // Check interval first to avoid starvation
                _ = interval.tick() => {
                    for download in self.all_downloads.iter_mut() {
                        download.update_progress().await;
                    }
                }

                // Process commands only if interval is not ready
                cmd = receiver.recv() => {
                    if let Some(cmd) = cmd {
                        self.handle_command(cmd).await;
                    } else {
                        // all senders are dropped
                        // continue because we need to update
                        continue;
                    }
                }
            }
        }
    }

    async fn handle_command(&mut self, command: ManagerCommand) {
        let respond_to = command.respond_to;
        let request_id = command.request.request_id;
        if let Some(req) = command.request.request {
            match req {
                Request::AddDownload(download_request) => {
                    let mut download = Download::new(
                        convert_to_download_req(download_request),
                        &DownloadConfig::default(),
                    );

                    let _ = respond_to.send(RpcResponse {
                        request_id,
                        response: Some(Response::DownloadCreated(GetDownload {
                            id: download.id.to_string(),
                        })),
                    });
                    let _ = download.start().await;
                    self.all_downloads.push(download);
                }
                Request::HeartBeat(heartbeat) => {
                    let _ = respond_to.send(RpcResponse {
                        request_id,
                        response: Some(Response::HearBeat(heartbeat)),
                    });
                }
                Request::GetDownload(request) => {
                    let id = request.id;
                    let download = self
                        .all_downloads
                        .iter()
                        .find(|s| s.id.to_string() == id)
                        .cloned();
                    match download {
                        Some(download) => {
                            let _ = respond_to.send(RpcResponse {
                                request_id,
                                response: Some(Response::Download(convert_to_download_proto(
                                    &download,
                                ))),
                            });
                        }
                        None => {
                            let _ = respond_to.send(RpcResponse {
                                request_id,
                                response: Some(Response::Error(ErrorProto {
                                    kind: "NOT FOUND".to_string(),
                                })),
                            });
                        }
                    }
                }
                Request::GetDownloads(_) => {
                    let downloads: Vec<utils::rpc_types::Download> = self
                        .all_downloads
                        .clone()
                        .iter()
                        .map(|d| convert_to_download_proto(d))
                        .collect();
                    let _ = respond_to.send(RpcResponse {
                        request_id,
                        response: Some(Response::Downloads(DownloadList { list: downloads })),
                    });
                }
            }
        }
    }
}
