use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum Message {
    Hello(String),
    Shutdown,
}

pub const SOCKET_ADDR: &str = "127.0.0.1:12345";
