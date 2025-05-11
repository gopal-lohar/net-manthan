use std::{path::PathBuf, time::Duration};

use colored::Colorize;
use download_engine::{
    Download, DownloadParts,
    download_config::DownloadConfig,
    types::{DownloadRequest, DownloadStatus},
    utils::format_bytes,
};
use tokio::{self, time::sleep};
use tracing::{Level, debug, error};
use utils::logging::{self, Component, LogConfig};

use clap::{ArgAction, Parser};

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

    let mut downloads: Vec<Download> = Vec::new();
    for url in cli.urls {
        let mut download = Download::new(
            DownloadRequest {
                url,
                file_dir: (&cli.dir).clone().unwrap_or("/tmp/".into()).into(),
                file_name: match &cli.out {
                    Some(out) => Some(out.into()),
                    None => None,
                },
                referrer: None,
                headers: None,
            },
            &DownloadConfig {
                connections_per_server: cli.split,
                ..Default::default()
            },
        );

        match download.load_download_info().await {
            Ok(_) => {}
            Err(e) => {
                error!("Failed to load download info: {}", e);
            }
        }

        // TODO: handle not loaded gracefully
        match download.start().await {
            Ok(_) => {}
            Err(e) => {
                error!("Failed to start download: {}", e);
            }
        }

        downloads.push(download);
    }

    println!("{}", "\n".repeat(4 * downloads.len()));

    loop {
        sleep(Duration::from_millis(250)).await;
        for download in &mut downloads {
            download.update_progress().await;
        }

        pretty_print_progress(&mut downloads);

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
}

fn pretty_print_progress(downloads: &mut Vec<Download>) {
    println!("\x1B[{}A", (downloads.len() * 4) + 2);
    for (index, download) in &mut downloads.iter_mut().enumerate() {
        let mut filename = download
            .file_name
            .clone()
            .unwrap_or(PathBuf::from("unnamed"))
            .to_string_lossy()
            .into_owned();
        filename = if filename.len() > 35 {
            format!("{}...", &filename[..32])
        } else {
            filename
        };
        let status = match download.get_status() {
            DownloadStatus::Downloading => "Downloading".blue(),
            DownloadStatus::Complete => "Complete".green(),
            DownloadStatus::Failed => "Failed".red(),
            DownloadStatus::Cancelled => "Cancelled".red(),
            _ => format!("{:?}", download.get_status()).red(),
        };
        println!(
            "\n\t\x1B[K {}. {} {}{}{}",
            index + 1,
            filename,
            " ".repeat(50 - (4 + filename.chars().count() + status.chars().count())),
            status,
            match download.last_update_time {
                Some(_) => "".into(),
                None => format!(" Average Speed: {}", download.get_formatted_average_speed()),
            }
        );
        let downloaded = format_bytes(download.get_bytes_downloaded());
        let total = format_bytes(download.get_total_size());
        let percentage = format!("{}%", download.get_progress_percentage() as usize,);
        let parts = match &download.parts {
            DownloadParts::NonResumable(_) => 1,
            DownloadParts::Resumable(p) => p.len(),
            DownloadParts::None => 0,
        }
        .to_string();
        let current_speed = format!("{}/s", format_bytes(download.get_current_speed() as u64));
        let eta = if matches!(download.get_status(), DownloadStatus::Complete) {
            "0s".to_string()
        } else if download.get_current_speed() == 0 {
            "∞".to_string()
        } else {
            let secs = (download.get_total_size() - download.get_bytes_downloaded())
                / (download.get_current_speed() as u64);
            format!("{}s", secs)
        };
        println!(
            "\t\x1B[K [{}/{}({}) Parts:{} Speed:{} ETA:{}]\r",
            downloaded,
            total,
            percentage.blue(),
            parts,
            current_speed.green(),
            eta.yellow(),
        );
        print_progress_string(download.get_progress_percentage(), 50);
    }
    println!("");
}

fn print_progress_string(progress: f64, width: usize) {
    let progress = if progress == 100.0 {
        100.0
    } else {
        progress % (100 as f64)
    };
    let green_bars = ((width as f64) * (progress / (100 as f64))).round() as usize;
    println!(
        "\t {}{}",
        "━".repeat(green_bars).green(),
        "━".repeat(width - green_bars).bright_black()
    )
}
