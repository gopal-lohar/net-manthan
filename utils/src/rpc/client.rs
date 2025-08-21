use std::sync::Arc;

use super::{
    RpcConfig,
    messages::{RpcRequest, RpcResponse},
    native_rpc_client::NativeRpcClient,
};
use crate::rpc::messages::{Message, Payload};
use anyhow::Result;
use tokio::sync::Mutex;

pub enum ClientRequest {
    Request(RpcRequest),
    Restart,
    Close,
}

pub enum ClientResponse {
    Response(RpcResponse),
    Acknowledge,
    NotConnected(String),
    Error(String),
}

pub enum RpcClient {
    Native(NativeRpcClient),
}

#[derive(Clone)]
pub struct Client {
    config: RpcConfig,
    secret: String,
    client: Arc<Mutex<Option<RpcClient>>>,
}

impl Client {
    pub fn new(secret: String, config: RpcConfig) -> Self {
        Self {
            config,
            secret,
            client: Arc::new(Mutex::new(None)),
        }
    }

    pub async fn connect(&self) -> Result<()> {
        let client = NativeRpcClient::connect(&self.config.native_rpc_settings).await?;
        let mut guard = self.client.lock().await;
        *guard = Some(RpcClient::Native(client));
        Ok(())
    }

    pub async fn send(&self, request: RpcRequest) -> Result<RpcResponse> {
        let mut client = self.client.lock().await;
        match client.as_mut() {
            Some(RpcClient::Native(client)) => match client
                .send_request(Message::request(request, self.secret.clone()))
                .await
            {
                Ok(response) => match response.payload {
                    Payload::Response(payload) => Ok(payload),
                    Payload::Request(_) => Err(anyhow::anyhow!("invalid payload payload received")),
                },
                Err(err) => Err(anyhow::anyhow!(err)),
            },
            None => Err(anyhow::anyhow!("Client not connected")),
        }
    }

    pub async fn close(self) -> Result<()> {
        let mut client = self.client.lock().await;
        match client.take() {
            Some(RpcClient::Native(client)) => client.close().await,
            None => Ok(()),
        }
    }
}
