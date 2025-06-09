use std::{path::PathBuf, time::Duration};

use crate::pretty_print_downloads::pretty_print_downloads;
use clap::{ArgAction, Parser};
use download_engine::{download_config::DownloadConfig, types::DownloadRequest};
use download_manager::DownloadManager;
use net_manthan_config::NetManthanConfig;
use tokio::{self, time::sleep};
use tracing::{Level, debug, error, info};
use utils::{
    conversion::{convert_from_download_proto, convert_to_download_req_proto},
    logging::{self, Component, LogConfig},
    rpc::ipc_server::RpcServer,
};

mod download_manager;
mod net_manthan_config;
mod pretty_print_downloads;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Set download directory
    #[arg(short = 'd', long = "dir", value_name = "DIR")]
    dir: Option<String>,

    /// Set output filename
    #[arg(short = 'o', long = "out", value_name = "FILE")]
    out: Option<String>,

    /// Download a file using N connections
    #[arg(short = 's', long = "split", value_name = "N", default_value = "10")]
    split: usize,

    /// Enable JSON-RPC/XML-RPC server
    #[arg(long = "enable-rpc", action = ArgAction::SetTrue)]
    enable_rpc: bool,

    /// Specify port for RPC server
    #[arg(long = "rpc-listen-port", value_name = "PORT", default_value = "6800")]
    rpc_port: u16,

    /// Set RPC secret authorization token
    #[arg(long = "rpc-secret", value_name = "TOKEN")]
    rpc_secret: Option<String>,

    /// Log file
    #[arg(short = 'l', long = "log", value_name = "LOG")]
    log: Option<String>,

    /// Set console log level
    #[arg(long = "console-log-level", value_name = "LEVEL",
          value_parser = ["trace", "debug", "info", "warn", "error"],
          default_value = "info")]
    log_level: String,

    /// URLs to download
    #[arg(required = true)]
    urls: Vec<String>,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let net_manthan_config = NetManthanConfig {
        download_dir: cli.dir,
        log_file: cli.log,
        log_level: cli.log_level,
        download_config: DownloadConfig::default(),
        max_concurrent_downloads: 10,
        ..Default::default()
    };

    // Initialize logging
    match logging::init_logging(LogConfig {
        component: Component::NetManthan,
        log_dir: net_manthan_config
            .log_file
            .map(PathBuf::from)
            .unwrap_or(".dev/logs".into()),
        silent_deps: vec!["hyper_util".into(), "mio".into()],
        max_level: match &net_manthan_config.log_level[..] {
            "trace" => Level::TRACE,
            "debug" => Level::DEBUG,
            "info" => Level::INFO,
            "warn" => Level::WARN,
            "error" => Level::ERROR,
            _ => {
                eprintln!(
                    "invalid log level in arguments, use of of the [\"trace\", \"debug\", \"info\", \"warn\", \"error\"]"
                );
                Level::INFO
            }
        },
        ..Default::default()
    }) {
        Ok(_) => {
            debug!("Logger initialized for {}", Component::NetManthan.as_str());
        }
        Err(e) => {
            eprintln!("Failed to initialize logger: {}", e);
        }
    }

    let mut manager_handle = DownloadManager::new();

    // if ipc is Disable it will be handled in the server only
    let ipc_server = RpcServer::new(&net_manthan_config.rpc_config, manager_handle.clone());
    ipc_server.start().await;

    for url in cli.urls {
        match manager_handle
            .add_download(convert_to_download_req_proto(DownloadRequest {
                url,
                file_dir: (&net_manthan_config.download_dir)
                    .clone()
                    .unwrap_or("/tmp/".into())
                    .into(),
                file_name: match &cli.out {
                    Some(out) => Some(out.into()),
                    None => None,
                },
                referrer: None,
                headers: None,
            }))
            .await
        {
            Ok(res) => {
                info!("Downlaod started for {}", res.id);
            }
            Err(err) => {
                error!("Something went wrong when adding download: {}", err);
            }
        }
    }

    let mut downloads: Vec<download_engine::Download>;

    loop {
        sleep(Duration::from_millis(500)).await;
        downloads = match manager_handle.get_downloads().await {
            Ok(downloads) => downloads
                .iter()
                .map(|d| convert_from_download_proto(d))
                .collect(),
            Err(e) => {
                error!("Failed to get downloads: {}", e);
                continue;
            }
        };

        pretty_print_downloads(&mut downloads, true);

        if downloads
            .iter()
            .all(|download| match download.get_status() {
                download_engine::types::DownloadStatus::Complete => true,
                _ => false,
            })
            && net_manthan_config.daemon
        {
            break;
        }
    }

    pretty_print_downloads(&mut downloads, false);
}
