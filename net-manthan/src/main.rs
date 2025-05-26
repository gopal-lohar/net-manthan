use std::time::Duration;

use crate::pretty_print_downloads::pretty_print_downloads;
use download_engine::types::DownloadRequest;
use download_manager::DownloadManager;
use tokio::{self, time::sleep};
use tracing::{Level, debug, error, info};
use utils::logging::{self, Component, LogConfig};

use clap::{ArgAction, Parser};

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
    // Initialize logging
    match logging::init_logging(LogConfig {
        component: Component::NetManthan,
        log_dir: ".dev/logs".into(),
        silent_deps: vec!["hyper_util".into(), "mio".into()],
        max_level: match cli.log_level.as_str() {
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

    let manager_handle = DownloadManager::new();

    for url in cli.urls {
        match manager_handle
            .add_download(DownloadRequest {
                url,
                file_dir: (&cli.dir).clone().unwrap_or("/tmp/".into()).into(),
                file_name: match &cli.out {
                    Some(out) => Some(out.into()),
                    None => None,
                },
                referrer: None,
                headers: None,
            })
            .await
        {
            Ok(id) => {
                info!("Downlaod started for {}", id.unwrap_or("default".into()));
            }
            Err(err) => {
                error!("Something went wrong when adding download: {}", err);
            }
        }
    }

    let mut downloads;

    loop {
        sleep(Duration::from_millis(500)).await;
        downloads = match manager_handle.get_downloads().await {
            Ok(downloads) => downloads,
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
        {
            break;
        }
    }

    pretty_print_downloads(&mut downloads, false);
}
