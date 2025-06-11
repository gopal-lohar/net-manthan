use crate::rpc::NativeRpcSettings;
use crate::rpc::message_codec::MessageCodec;
use crate::rpc_types::rpc_request::Request;
use crate::rpc_types::{RpcRequest, RpcResponse};
use anyhow::{Context, Result};
use bytes::{Buf, BytesMut};
use prost::Message;
use rand::Rng;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;
use tokio_util::codec::{Decoder, Encoder};
use tracing::debug;

/// Frame format: [length: u32][data: bytes]
/// This must match the server's MessageCodec exactly

#[derive(Debug)]
pub struct NativeRpcClient {
    stream: UnixStream,
    codec: MessageCodec,
    buffer: BytesMut,
}

impl NativeRpcClient {
    /// Connect to the Unix socket server
    pub async fn connect(settings: &NativeRpcSettings) -> Result<Self> {
        let stream = UnixStream::connect(&settings.address)
            .await
            .with_context(|| format!("Failed to connect to Unix socket: {}", settings.address))?;

        debug!("Connected to Unix socket: {}", settings.address);

        Ok(Self {
            stream,
            codec: MessageCodec,
            buffer: BytesMut::new(),
        })
    }

    /// Send a request and wait for response
    pub async fn send_request(&mut self, req: Request) -> Result<RpcResponse> {
        let mut rng = rand::rngs::ThreadRng::default();
        let request_id: u64 = rng.random();

        let request = RpcRequest {
            request_id,
            request: Some(req),
        };
        // Serialize the request
        let mut request_data = Vec::with_capacity(request.encoded_len());
        request
            .encode(&mut request_data)
            .context("Failed to encode request")?;

        // Encode with length prefix
        let mut encoded = BytesMut::new();
        self.codec
            .encode(request_data, &mut encoded)
            .context("Failed to encode message frame")?;

        // Send the request
        self.stream
            .write_all(&encoded)
            .await
            .context("Failed to write request to stream")?;

        debug!("Sent request: {}", request.request_id);

        // Read and decode response
        self.read_response().await
    }

    /// Read a response from the stream
    async fn read_response(&mut self) -> Result<RpcResponse> {
        loop {
            // Try to decode a complete message from buffer
            if let Some(message_data) = self
                .codec
                .decode(&mut self.buffer)
                .context("Failed to decode message frame")?
            {
                let response = RpcResponse::decode(message_data.chunk())
                    .context("Failed to deserialize response")?;

                debug!("Received response: {}", response.request_id);
                return Ok(response);
            }

            // Need more data, read from stream
            let mut temp = [0u8; 4096];
            match self.stream.read(&mut temp).await {
                Ok(0) => {
                    return Err(anyhow::anyhow!("Connection closed by server"));
                }
                Ok(n) => {
                    self.buffer.extend_from_slice(&temp[..n]);
                }
                Err(e) => {
                    return Err(anyhow::anyhow!("Failed to read from stream: {}", e));
                }
            }
        }
    }

    /// Close the connection
    pub async fn close(mut self) -> Result<()> {
        self.stream
            .shutdown()
            .await
            .context("Failed to shutdown stream")?;
        Ok(())
    }
}

pub async fn send_rpc_request(
    settings: &NativeRpcSettings,
    request: Request,
) -> Result<RpcResponse> {
    let mut client = NativeRpcClient::connect(settings).await?;
    let response = client.send_request(request).await?;
    client.close().await?;
    Ok(response)
}
