use download_engine::download_config::DownloadConfig;

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

#[derive(Debug, Clone)]
pub struct DownloadManagerConfig {
    daemon: bool,
    rpc_config: RpcConfig,
    conf_path: String,
    download_dir: String,
    log_file: String,
    log_level: String,
    max_concurrent_downloads: usize,
    download_config: DownloadConfig,
}
