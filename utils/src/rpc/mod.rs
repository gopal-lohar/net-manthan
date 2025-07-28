pub mod client;
pub mod message_codec;
pub mod messages;
pub mod native_rpc_client;
pub mod native_rpc_server;
pub mod server;

#[derive(Debug, Clone)]
pub struct NativeRpcSettings {
    /// In unix systems, whether to keep the socket permission user only
    pub allow_all_users: bool,
    /// Platform-specific address for local communication:
    /// - **Unix**: Filesystem path for the socket (e.g., `/tmp/myapp.sock`)
    /// - **Windows**: Named pipe identifier (e.g., `myapp-pipe`)
    pub address: String,
}

#[derive(Debug, Clone)]
pub struct RpcConfig {
    pub native_rpc_settings: NativeRpcSettings,
}
