use download_engine::download_config::DownloadConfig;
use utils::rpc::RpcConfig;

#[derive(Debug, Clone)]
pub struct NetManthanConfig {
    /// whether to close after downloads are done
    pub daemon: bool,
    /// configuration for RPC
    pub rpc_config: RpcConfig,
    /// default download directory
    pub download_dir: Option<String>,
    /// where to store logs
    pub log_file: Option<String>,
    /// log level in tracing
    pub log_level: String,
    /// number of downloads that can run cocurrently (not threads)
    #[allow(unused)]
    pub max_concurrent_downloads: usize,
    /// config for single download - comes from download_engine
    #[allow(unused)]
    pub download_config: DownloadConfig,
}

impl Default for NetManthanConfig {
    fn default() -> Self {
        Self {
            daemon: false,
            rpc_config: RpcConfig::Disabled,
            download_dir: None,
            log_file: None,
            log_level: "info".into(),
            max_concurrent_downloads: 10,
            download_config: DownloadConfig::default(),
        }
    }
}
