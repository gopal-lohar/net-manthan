use bincode;
use net_manthan_core::types::Message;
use std::io::prelude::*;
use std::net::TcpStream;


pub const IPC_SOCKET_ADDRESS: &str = "127.0.0.1:8080";

pub struct Client {
    stream: TcpStream,
}

impl Client {
    pub fn new(address: &str) -> std::io::Result<Self> {
        let stream = TcpStream::connect(address)?;
        Ok(Client { stream })
    }

    pub fn send_and_receive(&mut self, message: Message) -> std::io::Result<Message> {
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
