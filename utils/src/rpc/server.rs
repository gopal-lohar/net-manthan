use super::{
    RpcConfig,
    messages::{MAGIC_RESPONSE, Message, Payload, RpcRequest, RpcResponse},
    native_rpc_server::{NativeServerHandle, start_native_server},
};
use anyhow::Context;
use tokio::sync::{mpsc, oneshot};
use tracing::{error, info, warn};

pub struct ManagerCommand {
    pub request: RpcRequest,
    pub respond_to: oneshot::Sender<RpcResponse>,
}

impl ManagerCommand {
    pub async fn send(
        request: RpcRequest,
        sender: &mpsc::Sender<Self>,
    ) -> anyhow::Result<RpcResponse> {
        let (send, recv) = oneshot::channel();
        let command = Self {
            request,
            respond_to: send,
        };
        sender.send(command).await?;
        recv.await.context("Error Recieving the response")
    }

    pub async fn fire_forget(request: RpcRequest, sender: &mpsc::Sender<Self>) {
        let (send, _) = oneshot::channel();
        let command = Self {
            request,
            respond_to: send,
        };
        let _ = sender.send(command).await;
    }
}

pub struct RpcServer {
    config: RpcConfig,
    handle: RpcServerHandle,
    shutdown_handle: Option<NativeServerHandle>,
    task_handle: Option<tokio::task::JoinHandle<()>>,
}

impl RpcServer {
    pub fn new(config: &RpcConfig, handle: RpcServerHandle) -> Self {
        Self {
            config: config.clone(),
            handle,
            shutdown_handle: None,
            task_handle: None,
        }
    }

    pub async fn start(&mut self) {
        let settings = self.config.native_rpc_settings.clone();
        match start_native_server(self.handle.clone(), settings).await {
            Ok((shutdown_handle, task_handle)) => {
                info!("Native Rpc Server Started");
                self.shutdown_handle = Some(shutdown_handle);
                self.task_handle = Some(task_handle);
            }
            Err(_) => {
                error!("Failed to start Native Rpc Server");
            }
        }
    }

    pub async fn shutdown(mut self) {
        if let Some(shutdown_handle) = self.shutdown_handle.take() {
            let _ = shutdown_handle.shutdown().await;
        }
        if let Some(task_handle) = self.task_handle.take() {
            let _ = task_handle.await;
        }
    }
}

/// RpcServerHandle is a handle to the RpcServer. It handles encoding and decoding of requests and responses. into types.
#[derive(Debug, Clone)]
pub struct RpcServerHandle {
    pub command_sender: mpsc::Sender<ManagerCommand>,
    pub secret: String,
}

impl RpcServerHandle {
    pub async fn handle_call(&mut self, request_raw: &[u8]) -> Vec<u8> {
        let request = match Message::decode(request_raw) {
            Ok(request) => request,
            Err(_) => {
                return Message::new_response(
                    self.secret.clone(),
                    RpcResponse::Error("Malformed Request".to_string()),
                )
                .encode()
                .unwrap_or_else(|_| {
                    warn!("Failed to decode request");
                    Vec::from(MAGIC_RESPONSE)
                });
            }
        };
        let request_id = request.request_id;
        let (send, recv) = oneshot::channel();

        // check secret
        if request.secret != self.secret {
            return Message::response(
                request_id,
                self.secret.clone(),
                RpcResponse::Error("Invalid secret".to_string()),
            )
            .encode()
            .unwrap_or_else(|_| {
                warn!("Failed to encode response for request_id: {}", request_id);
                Vec::from(MAGIC_RESPONSE)
            });
        }

        let request = match request.payload {
            Payload::Request(request) => request,
            Payload::Response(_) => {
                return Message::response(
                    request_id,
                    self.secret.clone(),
                    RpcResponse::Error("Invalid payload".to_string()),
                )
                .encode()
                .unwrap_or_else(|_| {
                    warn!("Failed to encode response for request_id: {}", request_id);
                    Vec::from(MAGIC_RESPONSE)
                });
            }
        };

        // send request to manager
        if let Err(e) = self
            .command_sender
            .send(ManagerCommand {
                request,
                respond_to: send,
            })
            .await
        {
            return Message::response(
                request_id,
                self.secret.clone(),
                RpcResponse::Error(format!(
                    "Download manager thread has terminated. Error: {}",
                    e
                )),
            )
            .encode()
            .unwrap_or_else(|_| {
                warn!("Failed to encode response for request_id: {}", request_id);
                Vec::from(MAGIC_RESPONSE)
            });
        }

        match recv.await {
            Ok(res) => Message::response(request_id, self.secret.clone(), res)
                .encode()
                .unwrap_or_else(|_| {
                    warn!("Failed to encode response for request_id: {}", request_id);
                    Vec::from(MAGIC_RESPONSE)
                }),
            Err(e) => Message::response(
                request_id,
                self.secret.clone(),
                RpcResponse::Error(format!(
                    "Download manager dropped the response channel. Error: {}",
                    e
                )),
            )
            .encode()
            .unwrap_or(Vec::from(MAGIC_RESPONSE)),
        }
    }
}
