use bincode;
use download_engine::types::{IpcRequest, IpcResponse};
use std::io::prelude::*;
use std::net::TcpStream;

pub mod logging;

pub struct Client {
    stream: TcpStream,
}

impl Client {
    pub fn new(address: &str) -> std::io::Result<Self> {
        let stream = TcpStream::connect(address)?;
        Ok(Client { stream })
    }

    pub fn send_and_receive(&mut self, message: IpcRequest) -> std::io::Result<IpcResponse> {
        let serialized = bincode::serialize(&message)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

        // Send length prefix
        self.stream
            .write_all(&(serialized.len() as u64).to_le_bytes())?;
        // Send message
        self.stream.write_all(&serialized)?;
        self.stream.flush()?;

        // Read response length
        let mut len_bytes = [0u8; 8];
        self.stream.read_exact(&mut len_bytes)?;
        let msg_len = u64::from_le_bytes(len_bytes) as usize;

        let mut buffer = vec![0u8; msg_len];
        self.stream.read_exact(&mut buffer)?;
        bincode::deserialize(&buffer).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
    }
}

pub fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes < KB {
        format!("{} B", bytes)
    } else if bytes < MB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else if bytes < GB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    }
}
