use anyhow::{Context, Result};
use download_engine::{Download, download_config::DownloadConfig, types::DownloadRequest};
use tokio::{
    sync::{mpsc, oneshot},
    time::{Duration, interval},
};

pub enum ManagerCommand {
    AddDownload {
        request: DownloadRequest,
        respond_to: oneshot::Sender<Option<String>>,
    },
    #[allow(unused)]
    GetDownload {
        id: String,
        respond_to: oneshot::Sender<Option<Download>>,
    },
    GetDownloads {
        respond_to: oneshot::Sender<Vec<Download>>,
    },
}

/// Sender handle that can be cloned and shared with API servers
#[derive(Clone)]
pub struct DownloadManagerHandle {
    command_sender: mpsc::Sender<ManagerCommand>,
}

impl DownloadManagerHandle {
    pub async fn add_download(&self, request: DownloadRequest) -> Result<Option<String>> {
        let (send, recv) = oneshot::channel();
        self.command_sender
            .send(ManagerCommand::AddDownload {
                request,
                respond_to: send,
            })
            .await
            .context("Download manager thread has terminated")?;

        recv.await
            .context("Download manager dropped the response channel")
    }

    #[allow(unused)]
    pub async fn get_download(&self, id: String) -> Result<Option<Download>> {
        let (send, recv) = oneshot::channel();
        self.command_sender
            .send(ManagerCommand::GetDownload {
                id,
                respond_to: send,
            })
            .await
            .context("Download manager thread has terminated")?;

        recv.await
            .context("Download manager dropped the response channel")
    }

    pub async fn get_downloads(&self) -> Result<Vec<Download>> {
        let (send, recv) = oneshot::channel();
        self.command_sender
            .send(ManagerCommand::GetDownloads { respond_to: send })
            .await
            .context("Download manager thread has terminated")?;

        recv.await
            .context("Download manager dropped the response channel")
    }
}

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

    async fn handle_command(&mut self, cmd: ManagerCommand) {
        match cmd {
            ManagerCommand::AddDownload {
                request,
                respond_to,
            } => {
                let mut download = Download::new(request, &DownloadConfig::default());
                match respond_to.send(Some(download.id.to_string())) {
                    Ok(_) => {}
                    Err(_) => {}
                }
                match download.start().await {
                    Ok(_) => {}
                    Err(_) => {}
                }
                self.all_downloads.push(download);
            }

            ManagerCommand::GetDownload { id, respond_to } => {
                let download = self
                    .all_downloads
                    .iter()
                    .find(|s| s.id.to_string() == id)
                    .cloned();
                match respond_to.send(download) {
                    Ok(_) => {}
                    Err(_) => {}
                }
            }

            ManagerCommand::GetDownloads { respond_to } => {
                match respond_to.send(self.all_downloads.clone()) {
                    Ok(_) => {}
                    Err(_) => {}
                }
            }
        }
    }
}
