use anyhow::Result;
use bincode::{deserialize, serialize};
use crossbeam_channel::{bounded, Receiver, Sender};
use net_manthan_core::types::{IpcRequest, IpcResponse};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;

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
                    thread::spawn(move || {
                        handle_client(stream, response_rx, request_tx);
                    });
                }
                Err(e) => {
                    eprintln!("Connection failed: {}", e);
                }
            }
        }
    });

    Ok((ipc_response_sender, ipc_request_receiver))
}

// TODO: use the freaking logs not eprintln! and stuff
fn handle_client(
    mut stream: TcpStream,
    ipc_response_receiver: Receiver<IpcResponse>,
    ipc_request_sender: Sender<IpcRequest>,
) {
    let mut buffer = [0; 1024]; // TODO: change buffer size?
    stream
        .set_read_timeout(Some(std::time::Duration::from_secs(30)))
        .expect("Failed to set read timeout"); // Read timeout

    while let Ok(n) = stream.read(&mut buffer) {
        if n == 0 {
            // Connection closed
            break;
        }

        match deserialize::<IpcRequest>(&buffer[0..n]) {
            Ok(request) => {
                if let Err(e) = ipc_request_sender.send(request) {
                    eprintln!("Failed to forward IPC request: {}", e);
                    break;
                }

                match ipc_response_receiver.recv() {
                    Ok(response) => match serialize(&response) {
                        Ok(data) => {
                            if let Err(e) = stream.write_all(&data) {
                                eprintln!("Failed to send response: {}", e);
                                break;
                            }
                        }
                        Err(e) => {
                            eprintln!("Failed to serialize response: {}", e);
                            break;
                        }
                    },
                    Err(e) => {
                        eprintln!("Failed to receive response: {}", e);
                        break;
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to deserialize request: {}", e);
                break;
            }
        }
    }
}
