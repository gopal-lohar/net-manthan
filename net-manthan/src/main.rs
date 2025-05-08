use std::{f32::consts::PI, path::PathBuf, time::Duration};

use colored::Colorize;
use download_engine::{
    Download, DownloadParts, download_config::DownloadConfig, types::DownloadRequest,
    utils::format_bytes,
};
use tokio::{self, time::sleep};
use tracing::{Level, debug, error, info};
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
    #[arg(short = 's', long = "split", value_name = "N", default_value = "3")]
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
            Ok(_) => {
                info!("Download info loaded successfully");
            }
            Err(e) => {
                error!("Failed to load download info: {}", e);
            }
        }

        // TODO: handle not loaded gracefully
        match download.start().await {
            Ok(_) => {
                // info!("Download started successfully");
            }
            Err(e) => {
                error!("Failed to start download: {}", e);
            }
        }

        downloads.push(download);
    }

    println!("{:?}", downloads);

    println!("");

    loop {
        sleep(Duration::from_millis(250)).await;

        for download in &mut downloads {
            download.update_progress().await;
        }

        let download = &downloads[0];
        match &download.parts {
            DownloadParts::Resumable(parts) => {
                // for part in parts {
                //     println!(
                //         "{}-{} {}/{}",
                //         part.start_byte,
                //         part.end_byte,
                //         part.bytes_downloaded,
                //         part.get_total_size()
                //     )
                // }
            }
            _ => {}
        }

        // pretty_print_progress(&mut downloads);
    }
}

fn pretty_print_progress(downloads: &mut Vec<Download>) {
    for (index, download) in &mut downloads.iter_mut().enumerate() {
        println!(
            "\t\x1B[K {}. {:?}    {:?}",
            index + 1,
            &download
                .file_name
                .clone()
                .unwrap_or(PathBuf::from("unnamed")),
            download.get_status()
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
        let eta = if download.get_current_speed() == 0 {
            "∞".to_string()
        } else {
            let secs = (download.get_total_size() - download.get_bytes_downloaded())
                / (download.get_current_speed() as u64);
            format!("{}S", secs)
        };
        println!(
            "\t\x1B[K [{}/{}({}) Parts:{} Speed:{} ETA:{} details: {}]\r",
            downloaded,
            total,
            percentage,
            parts,
            current_speed,
            eta,
            match &download.parts {
                DownloadParts::NonResumable(part) => {
                    format!("{}/{}", part.bytes_downloaded, part.total_size)
                }
                DownloadParts::Resumable(parts) => {
                    parts
                        .iter()
                        .map(|part| {
                            format!(" {}/{} ", part.bytes_downloaded, part.get_total_size())
                        })
                        .collect::<String>()
                }
                DownloadParts::None => {
                    "".into()
                }
            }
        );
        print_progress_string(download.get_progress_percentage(), 50);
        println!("\n\n\n");
    }
    println!("");
    println!("\x1B[{}A", (downloads.len() * 4) + 4);
}

fn print_progress_string(mut progress: f64, width: usize) {
    progress = progress % (100 as f64);
    let green_bars = ((width as f64) * (progress / (100 as f64))).round() as usize;
    println!(
        "\t {}{}",
        "━".repeat(green_bars).green(),
        "━".repeat(width - green_bars).bright_black()
    )
}
