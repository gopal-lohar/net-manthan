use download_engine::download_config::DownloadConfig;

#[derive(Debug, Clone)]
pub struct RpcSettings {
    pub listen_all: bool,
    pub allow_origin_all: bool,
    pub listen_port: u16,
    pub secret: String,
}

#[derive(Debug, Clone)]
pub enum RpcConfig {
    Disabled,
    Grpc(RpcSettings),
    JsonRpc(RpcSettings),
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

// impl Default for DownloadManagerConfig {
//     fn default() -> Self {
//         Self { daemon: false, rpc_config: RpcConfig::Disabled }
//     }
// }
