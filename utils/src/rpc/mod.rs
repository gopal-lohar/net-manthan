pub mod client;
pub mod message_codec;
pub mod server;
use std::fmt;

#[derive(Debug, Clone)]
pub struct RpcSettings {
    pub listen_all: bool,
    pub allow_origin_all: bool,
    pub listen_port: u16,
    pub secret: String,
}

#[derive(Debug, Clone)]
pub struct NativeRpcSettings {
    /// Platform-specific address for local communication:
    /// - **Unix**: Filesystem path for the socket (e.g., `/tmp/myapp.sock`)
    /// - **Windows**: Named pipe identifier (e.g., `myapp-pipe`)
    pub address: String,
    pub secret: String,
    pub allow_all_users: bool,
}

#[derive(Debug, Clone)]
pub enum RpcConfig {
    // the details config is called settings to avaoid ambiguity
    Disabled,
    Grpc(RpcSettings),
    JsonRpc(RpcSettings),
    Native(NativeRpcSettings),
}

impl fmt::Display for RpcConfig {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RpcConfig::Disabled => write!(f, "Disabled"),
            RpcConfig::Grpc(_) => write!(f, "Grpc"),
            RpcConfig::JsonRpc(_) => write!(f, "JsonRpc"),
            RpcConfig::Native(_) => write!(f, "Native"),
        }
    }
}
