use anyhow::Result;
use bincode::{deserialize, serialize};
use crossbeam_channel::{bounded, Receiver, Sender};
use download_engine::types::{IpcRequest, IpcResponse};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;
use tracing::{debug, error, info};

// TODO: use tokio tasks
// TODO: this has some race conditions going on but who cares?
pub fn start_ipc_server(address: &String) -> Result<(Sender<IpcResponse>, Receiver<IpcRequest>)> {
    let (ipc_request_sender, ipc_request_receiver) = bounded::<IpcRequest>(100);
    let (ipc_response_sender, ipc_response_receiver) = bounded::<IpcResponse>(100);

    let address_clone = address.clone();

    let listener = TcpListener::bind(&address_clone)?;

    // Spawn TCP server thread
    thread::spawn(move || {
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let response_rx = ipc_response_receiver.clone();
                    let request_tx = ipc_request_sender.clone();
                    let peer_addr = match stream.peer_addr() {
                        Ok(addr) => addr.to_string(),
                        Err(_) => "Unknown Address".to_string(),
                    };

                    info!("A client connected with addr - {}", peer_addr);
                    thread::spawn(move || {
                        handle_client(stream, response_rx, request_tx);
                    });
                }
                Err(e) => {
                    error!("Connection failed: {}", e);
                }
            }
        }
    });

    Ok((ipc_response_sender, ipc_request_receiver))
}

fn handle_client(
    mut stream: TcpStream,
    ipc_response_receiver: Receiver<IpcResponse>,
    ipc_request_sender: Sender<IpcRequest>,
) {
    let mut length_buffer = [0u8; 8];
    while let Ok(_) = stream.read_exact(&mut length_buffer) {
        let message_length = u64::from_le_bytes(length_buffer) as usize;

        let mut message_buffer = vec![0u8; message_length];

        debug!("length of message received: {}", message_length);

        match stream.read_exact(&mut message_buffer) {
            Ok(_) => match deserialize::<IpcRequest>(&message_buffer) {
                Ok(request) => {
                    info!("IPC message received from the client: {:?}", request);
                    if let Err(e) = ipc_request_sender.send(request) {
                        error!("Failed to forward IPC request: {}", e);
                        break;
                    }

                    match ipc_response_receiver.recv() {
                        Ok(response) => match serialize(&response) {
                            Ok(data) => {
                                if let Err(e) = stream.write_all(&(data.len() as u64).to_le_bytes())
                                {
                                    error!("Failed to send response length: {}", e);
                                    break;
                                };
                                if let Err(e) = stream.write_all(&data) {
                                    error!("Failed to send response: {}", e);
                                    break;
                                }
                                if let Err(e) = stream.flush() {
                                    error!("Failed to send response (while flushing): {}", e);
                                    break;
                                } else {
                                    info!("Response sent to client successfully: {:?}", response);
                                }
                            }
                            Err(e) => {
                                error!("Failed to serialize response: {}", e);
                                break;
                            }
                        },
                        Err(e) => {
                            error!("Failed to receive response: {}", e);
                            break;
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to deserialize request: {}", e);
                    break;
                }
            },
            Err(e) => {
                if e.kind() == std::io::ErrorKind::UnexpectedEof {
                    debug!("Client disconnected");
                } else {
                    error!("Error reading length prefix: {}", e);
                }
            }
        }
    }
}
