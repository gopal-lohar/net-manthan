use crate::{
    rpc::{
        RpcConfig,
        server::native::{NativeServerHandle, start_native_server},
    },
    rpc_types::{
        Download, DownloadRequest, Error, GetDownload, GetDownloads, RpcRequest, RpcResponse,
        rpc_request::Request, rpc_response::Response,
    },
};
use rand::Rng;
use tokio::sync::{mpsc, oneshot};
use tracing::{error, info};

mod native;

pub struct ManagerCommand {
    pub request: RpcRequest,
    pub respond_to: oneshot::Sender<RpcResponse>,
}

pub struct RpcServer {
    config: RpcConfig,
    handle: RpcServerHandle,
    // later it can become an enum for all other servers,or just take a onshot channel
    shutdown_handle: Option<NativeServerHandle>,
}

impl RpcServer {
    pub fn new(config: &RpcConfig, handle: RpcServerHandle) -> Self {
        Self {
            config: config.clone(),
            handle,
            shutdown_handle: None,
        }
    }

    pub async fn start(&mut self) {
        match &self.config {
            RpcConfig::Disabled => {
                info!("RPC disabled");
            }
            RpcConfig::Native(settings) => {
                match start_native_server(self.handle.clone(), settings.clone()).await {
                    Ok(h) => {
                        info!("Native Rpc Server Started");
                        self.shutdown_handle = Some(h);
                    }
                    Err(_) => {
                        error!("Failed to start Native Rpc Server");
                    }
                }
            }
            _ => {
                todo!();
            }
        }
    }

    pub async fn shutdown(self) {
        match self.shutdown_handle {
            Some(h) => {
                let _ = h.shutdown().await;
            }
            None => {}
        }
    }
}

#[derive(Debug, Clone)]
pub struct RpcServerHandle {
    pub command_sender: mpsc::Sender<ManagerCommand>,
}

impl RpcServerHandle {
    pub async fn handle_call(&mut self, request: RpcRequest) -> RpcResponse {
        let request_id = request.request_id;
        let (send, recv) = oneshot::channel();
        if let Err(e) = self
            .command_sender
            .send(ManagerCommand {
                request,
                respond_to: send,
            })
            .await
        {
            return RpcResponse {
                request_id: request_id,
                response: Some(Response::Error(Error {
                    kind: format!("Download manager thread has terminated. Error: {}", e),
                })),
            };
        }

        match recv.await {
            Ok(res) => res,
            Err(e) => RpcResponse {
                request_id: request_id,
                response: Some(Response::Error(Error {
                    kind: format!(
                        "Download manager dropped the response channel. Error: {}",
                        e
                    ),
                })),
            },
        }
    }

    pub async fn add_download(&mut self, request: DownloadRequest) -> Result<GetDownload, String> {
        let mut rng = rand::rngs::ThreadRng::default();
        let request_id: u64 = rng.random();

        let response = self
            .handle_call(RpcRequest {
                request_id,
                request: Some(Request::AddDownload(request)),
            })
            .await;

        match response.response {
            Some(res) => match res {
                Response::DownloadCreated(download) => Ok(download),
                _ => Err("Something went wrong".to_string()),
            },
            None => Err("Something went wrong".to_string()),
        }
    }

    pub async fn get_downloads(&mut self) -> Result<Vec<Download>, String> {
        let mut rng = rand::rngs::ThreadRng::default();
        let request_id: u64 = rng.random();

        let response = self
            .handle_call(RpcRequest {
                request_id,
                request: Some(Request::GetDownloads(GetDownloads {})),
            })
            .await;

        match response.response {
            Some(res) => match res {
                Response::Downloads(d) => Ok(d.list),
                _ => Err("Something went wrong".to_string()),
            },
            None => Err("Something went wrong".to_string()),
        }
    }
}
