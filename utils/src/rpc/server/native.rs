use crate::rpc::NativeRpcSettings;
use crate::rpc::message_codec::MessageCodec;
use crate::rpc::server::RpcServerHandle;
use crate::rpc_types::RpcRequest;
use anyhow::Result;
use bytes::{Buf, BytesMut};
use prost::Message;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::oneshot;
use tokio_util::codec::{Decoder, Encoder};
use tracing::{debug, error, info};

#[cfg(windows)]
use tokio::net::windows::named_pipe::{NamedPipeServer, ServerOptions};

#[cfg(unix)]
use anyhow::Context;
#[cfg(unix)]
use tokio::net::{UnixListener, UnixStream};
#[cfg(unix)]
use tracing::warn;

#[derive(Debug)]
pub struct NativeServerHandle {
    shutdown_tx: oneshot::Sender<()>,
}

impl NativeServerHandle {
    pub async fn shutdown(self) -> Result<()> {
        let _ = self.shutdown_tx.send(());
        Ok(())
    }
}

pub async fn start_native_server(
    handler: RpcServerHandle,
    settings: NativeRpcSettings,
) -> Result<NativeServerHandle> {
    let (shutdown_tx, shutdown_rx) = oneshot::channel();

    #[cfg(unix)]
    {
        start_unix_server(handler, settings, shutdown_rx).await?;
    }

    #[cfg(windows)]
    {
        start_windows_server(handler, settings, shutdown_rx).await?;
    }

    #[cfg(not(any(unix, windows)))]
    {
        anyhow::bail!("Native IPC not supported on this platform");
    }

    Ok(NativeServerHandle { shutdown_tx })
}

#[cfg(unix)]
async fn start_unix_server(
    handler: RpcServerHandle,
    settings: NativeRpcSettings,
    mut shutdown_rx: oneshot::Receiver<()>,
) -> Result<()> {
    // Clean up existing socket file
    if std::path::Path::new(&settings.address).exists() {
        std::fs::remove_file(&settings.address)
            .with_context(|| format!("Failed to remove existing socket: {}", settings.address))?;
    }

    let listener = UnixListener::bind(&settings.address)
        .with_context(|| format!("Failed to bind Unix socket: {}", settings.address))?;

    // Set permissions
    if !settings.allow_all_users {
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::Permissions::from_mode(0o600); // Owner only
        std::fs::set_permissions(&settings.address, perms)
            .with_context(|| format!("Failed to set socket permissions: {}", settings.address))?;
    }

    info!(
        "Native RPC server listening on Unix socket: {}",
        settings.address
    );

    tokio::spawn(async move {
        loop {
            tokio::select! {
                result = listener.accept() => {
                    match result {
                        Ok((stream, _)) => {
                            let handler = handler.clone();
                            tokio::spawn(async move {
                                if let Err(e) = handle_unix_connection(stream, handler).await {
                                    error!("Error handling Unix connection: {}", e);
                                }
                            });
                        }
                        Err(e) => {
                            error!("Failed to accept Unix connection: {}", e);
                        }
                    }
                }
                _ = &mut shutdown_rx => {
                    info!("Shutting down Unix socket server");
                    break;
                }
            }
        }

        // Clean up socket file
        if let Err(e) = std::fs::remove_file(&settings.address) {
            warn!("Failed to remove socket file on shutdown: {}", e);
        }
    });

    Ok(())
}

#[cfg(unix)]
async fn handle_unix_connection(
    mut stream: UnixStream,
    mut handler: RpcServerHandle,
) -> Result<()> {
    let mut buffer = BytesMut::new();
    let mut codec = MessageCodec;

    loop {
        // Read data from stream
        let mut temp = [0u8; 4096];
        match stream.read(&mut temp).await {
            Ok(0) => break, // Connection closed
            Ok(n) => {
                buffer.extend_from_slice(&temp[..n]);
            }
            Err(e) => {
                error!("Failed to read from Unix stream: {}", e);
                break;
            }
        }

        // Process complete messages
        while let Some(message_data) = codec.decode(&mut buffer)? {
            let request: RpcRequest = match RpcRequest::decode(message_data.chunk()) {
                Ok(req) => req,
                Err(e) => {
                    error!("Failed to deserialize request: {}", e);
                    continue;
                }
            };

            debug!("Received request: {}", request.request_id);

            let response = handler.handle_call(request).await;
            let mut response_data = Vec::with_capacity(response.encoded_len());
            if let Err(e) = response.encode(&mut response_data) {
                error!("Failed to encode response into bytes: {}", e);
                break;
            }

            let mut encoded = BytesMut::new();
            codec.encode(response_data, &mut encoded)?;

            if let Err(e) = stream.write_all(&encoded).await {
                error!("Failed to write response to Unix stream: {}", e);
                break;
            }
        }
    }

    Ok(())
}

#[cfg(windows)]
async fn start_windows_server(
    handler: RpcServerHandle,
    settings: NativeRpcSettings,
    mut shutdown_rx: oneshot::Receiver<()>,
) -> Result<()> {
    let pipe_name = format!(r"\\.\pipe\{}", settings.address);

    info!("Native RPC server listening on named pipe: {}", pipe_name);

    tokio::spawn(async move {
        loop {
            // Create server options for the named pipe
            let server_options = ServerOptions::new();

            // Create the named pipe server
            let server = match server_options.create(&pipe_name) {
                Ok(server) => server,
                Err(e) => {
                    error!("Failed to create named pipe server: {}", e);
                    break;
                }
            };

            tokio::select! {
                result = server.connect() => {
                    match result {
                        Ok(()) => {
                            let handler = handler.clone();
                            tokio::spawn(async move {
                                if let Err(e) = handle_windows_connection(server, handler).await {
                                    error!("Error handling named pipe connection: {}", e);
                                }
                            });
                        }
                        Err(e) => {
                            error!("Failed to connect to named pipe client: {}", e);
                        }
                    }
                }
                _ = &mut shutdown_rx => {
                    info!("Shutting down named pipe server");
                    break;
                }
            }
        }
    });

    Ok(())
}

#[cfg(windows)]
async fn handle_windows_connection(
    mut pipe: NamedPipeServer,
    mut handler: RpcServerHandle,
) -> Result<()> {
    let mut buffer = BytesMut::new();
    let mut codec = MessageCodec;

    loop {
        // Read data from named pipe
        let mut temp = [0u8; 4096];
        match pipe.read(&mut temp).await {
            Ok(0) => break, // Connection closed
            Ok(n) => {
                buffer.extend_from_slice(&temp[..n]);
            }
            Err(e) => {
                error!("Failed to read from named pipe: {}", e);
                break;
            }
        }

        // Process complete messages
        while let Some(message_data) = codec.decode(&mut buffer)? {
            let request: RpcRequest = match RpcRequest::decode(message_data.chunk()) {
                Ok(req) => req,
                Err(e) => {
                    error!("Failed to deserialize request: {}", e);
                    continue;
                }
            };

            debug!("Received request: {}", request.request_id);

            let response = handler.handle_call(request).await;
            let mut response_data = Vec::with_capacity(response.encoded_len());
            if let Err(e) = response.encode(&mut response_data) {
                error!("Failed to encode response into bytes: {}", e);
                break;
            }

            let mut encoded = BytesMut::new();
            codec.encode(response_data, &mut encoded)?;

            if let Err(e) = pipe.write_all(&encoded).await {
                error!("Failed to write response to named pipe: {}", e);
                break;
            }
        }
    }

    Ok(())
}
