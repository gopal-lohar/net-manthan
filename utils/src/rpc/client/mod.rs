use crate::conversion::convert_from_download_proto;
use crate::rpc::NativeRpcSettings;
use crate::rpc::message_codec::MessageCodec;
use crate::rpc_types::rpc_request::Request;
use crate::rpc_types::{GetDownloads, RpcRequest, RpcResponse, rpc_response};
use anyhow::{Context, Result};
use bytes::{Buf, BytesMut};
use download_engine::Download;
use prost::Message;
use rand::random;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_util::codec::{Decoder, Encoder};
use tracing::debug;

#[cfg(unix)]
use tokio::net::UnixStream;

#[cfg(windows)]
use tokio::net::windows::named_pipe::{ClientOptions, NamedPipeClient};

/// Frame format: [length: u32][data: bytes]
/// This must match the server's MessageCodec exactly

#[derive(Debug)]
pub struct NativeRpcClient {
    #[cfg(unix)]
    stream: UnixStream,
    #[cfg(windows)]
    pipe: NamedPipeClient,
    codec: MessageCodec,
    buffer: BytesMut,
}

impl NativeRpcClient {
    /// Connect to the native server (Unix socket or Windows named pipe)
    pub async fn connect(settings: &NativeRpcSettings) -> Result<Self> {
        #[cfg(unix)]
        {
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

        #[cfg(windows)]
        {
            let pipe_name = format!(r"\\.\pipe\{}", settings.address);
            let pipe = ClientOptions::new()
                .open(&pipe_name)
                .with_context(|| format!("Failed to connect to named pipe: {}", pipe_name))?;

            debug!("Connected to named pipe: {}", pipe_name);

            Ok(Self {
                pipe,
                codec: MessageCodec,
                buffer: BytesMut::new(),
            })
        }

        #[cfg(not(any(unix, windows)))]
        {
            anyhow::bail!("Native IPC not supported on this platform");
        }
    }

    /// Send a request and wait for response
    pub async fn send_request(&mut self, req: Request) -> Result<RpcResponse> {
        let request_id = random::<u64>();

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
        self.write_all(&encoded).await
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
            match self.read(&mut temp).await {
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

    /// Platform-agnostic read implementation
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, std::io::Error> {
        #[cfg(unix)]
        {
            self.stream.read(buf).await
        }
        #[cfg(windows)]
        {
            self.pipe.read(buf).await
        }
    }

    /// Platform-agnostic write_all implementation
    async fn write_all(&mut self, buf: &[u8]) -> Result<(), std::io::Error> {
        #[cfg(unix)]
        {
            self.stream.write_all(buf).await
        }
        #[cfg(windows)]
        {
            self.pipe.write_all(buf).await
        }
    }

    /// Close the connection
    pub async fn close(mut self) -> Result<()> {
        #[cfg(unix)]
        {
            self.stream
                .shutdown()
                .await
                .context("Failed to shutdown stream")?;
        }
        #[cfg(windows)]
        {
            // Named pipes don't require explicit shutdown
            // The connection will be closed when the client is dropped
        }
        Ok(())
    }
}

impl NativeRpcClient {
    pub async fn get_downloads(&mut self) -> Result<Vec<Download>> {
        let request = Request::GetDownloads(GetDownloads {});
        let response = self.send_request(request).await?;
        match response.response {
            Some(response) => match response {
                rpc_response::Response::Downloads(downloads) => Ok(downloads
                    .list
                    .iter()
                    .map(|d| convert_from_download_proto(d))
                    .collect()),
                _ => Err(anyhow::anyhow!("Unexpected response")),
            },
            None => Err(anyhow::anyhow!("Failed to get downloads")),
        }
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