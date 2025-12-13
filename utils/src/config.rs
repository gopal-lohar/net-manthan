use clap::{builder::BoolishValueParser, Args};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, Args)]
pub struct Config {
    #[clap(flatten)]
    pub vayuget: Vayuget,
    #[clap(flatten)]
    pub rpc: RpcConfig,
}

impl Default for Config {
    fn default() -> Self {
        let db_path = dirs::data_dir()
            .map(|p| p.join("net-manthan").join("vayu.db"))
            .unwrap();

        let downloads_path = dirs::download_dir().unwrap();

        let log_path = dirs::data_dir()
            .map(|p| p.join("net-manthan").join("vayuget.log"))
            .unwrap();

        let vayuget = Vayuget {
            db_path,
            downloads_path,
            log_path,
            log_level: "info".to_string(),
            pretty_print: None,
        };

        // Platform-specific address for local communication:
        // - **Unix**: Filesystem path for the socket (e.g., `/tmp/myapp.sock`)
        // - **Windows**: Named pipe identifier (e.g., `myapp-pipe`)
        let address = if cfg!(windows) {
            "vayu-pipe".to_string()
        } else {
            "/tmp/vayu.sock".to_string()
        };
        let native_rpc_settings = NativeRpcSettings {
            allow_all_users: true,
            address,
            rpc_secret: None,
        };

        let rpc = RpcConfig {
            native_rpc_settings,
        };

        Self { vayuget, rpc }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Args)]
#[command(author, version, about, long_about = None)]
pub struct Vayuget {
    /// Path to the database file
    #[arg(long, default_value_os_t = default_db_path())]
    pub db_path: PathBuf,
    /// Path to the downloads directory
    #[arg(long, default_value_os_t = default_downloads_path())]
    pub downloads_path: PathBuf,
    /// Path to the log file
    #[arg(long, default_value_os_t = default_log_path())]
    pub log_path: PathBuf,
    /// Set logging verbosity level
    #[arg(
        long,
        value_name = "LEVEL",
        value_parser = ["trace", "debug", "info", "warn", "error"],
        default_value = "info"
    )]
    pub log_level: String,
    /// Control pretty printing (default: true unless --daemon is used)
    #[arg(
        long,
        action = clap::ArgAction::Set,
        num_args = 0..=1,
        require_equals = true,
        default_missing_value = "true",
        value_parser = BoolishValueParser::new(),
    )]
    pub pretty_print: Option<bool>,
}

fn default_db_path() -> PathBuf {
    dirs::data_dir()
        .map(|p| p.join("net-manthan").join("vayu.db"))
        .unwrap()
}

fn default_downloads_path() -> PathBuf {
    dirs::download_dir().unwrap()
}

fn default_log_path() -> PathBuf {
    dirs::data_dir()
        .map(|p| p.join("net-manthan").join("vayuget.log"))
        .unwrap()
}

#[derive(Debug, Clone, Serialize, Deserialize, Args)]
pub struct NativeRpcSettings {
    /// In unix systems, whether to keep the socket permission user only
    #[arg(long, default_value_t = true)]
    pub allow_all_users: bool,
    /// Platform-specific address for local communication:
    /// - **Unix**: Filesystem path for the socket (e.g., `/tmp/myapp.sock`)
    /// - **Windows**: Named pipe identifier (e.g., `myapp-pipe`)
    #[arg(long, default_value_t = default_address())]
    pub address: String,
    /// Authorization token for RPC access
    #[arg(long, value_name = "TOKEN")]
    pub rpc_secret: Option<String>,
}

fn default_address() -> String {
    if cfg!(windows) {
        "vayu-pipe".to_string()
    } else {
        "/tmp/vayu.sock".to_string()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Args)]
pub struct RpcConfig {
    #[clap(flatten)]
    pub native_rpc_settings: NativeRpcSettings,
}
