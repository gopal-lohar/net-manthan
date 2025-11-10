// This binary is not an application, it is something that takes arguments and works on the basis of that, no management of any db, logs or cache. the application part will be handled my ui.

use clap::{ArgAction, Parser, builder::BoolishValueParser};
use download_manager::DownloadManager;
use engine::types::{
    messages::DownloadRequestMessage,
    request::{DownloadRequest, Headers},
};
use sqlx::SqlitePool;
use std::fs;
use tokio::sync::mpsc;
use tracing::Level;
use utils::{
    logger::{Component, LogConfig, get_engine_silent_deps, init_logger},
    rpc::{messages::RpcRequest, server::ManagerCommand},
};

pub mod ctrl_c;
pub mod download_manager;

#[derive(Parser)]
#[command(
    author,
    version,
    name = "vayu",
    about = "Multithreaded download manager backend",
    long_about = "Vayu handles download operations either as:
  - Direct CLI tool: Downloads specified URLs immediately
  - Background service: Runs as daemon with RPC interface for management

Note: Database and log directories must exist when specified"
)]
pub struct Cli {
    /// Run as a background daemon with RPC server
    #[arg(
        long = "daemon",
        action = ArgAction::SetTrue,
        help = "Enables background service mode with RPC server (Unix sockets/Named pipes)"
    )]
    daemon: bool,

    /// Control pretty printing (default: true unless --daemon is used)
    #[arg(
        long = "pretty-print",
        action = ArgAction::Set,
        num_args = 0..=1,         // Accepts 0 or 1 arguments
        require_equals = true,     // Requires '=' for values
        default_missing_value = "true", // --pretty-print => true
        value_parser = BoolishValueParser::new(),
        help = "Pretty-print output [auto: !daemon, allow: true|false]"
    )]
    pretty_print: Option<bool>,

    /// Authorization token for RPC access
    #[arg(
        long = "rpc-secret",
        value_name = "TOKEN",
        help = "Secures RPC communication [default: no authentication]"
    )]
    rpc_secret: Option<String>,

    /// Directory for storing the download file in non-daemon mode
    #[arg(
        short = 'd',
        long = "dir",
        value_name = "PATH",
        default_value = ".",
        help = "Log directory [required for file logging]"
    )]
    dir: String,

    /// Set logging verbosity level
    #[arg(
        long = "log-level",
        value_name = "LEVEL",
        value_parser = ["trace", "debug", "info", "warn", "error"],
        default_value = "info",
        help = "Log detail level (trace|debug|info|warn|error)"
    )]
    log_level: String,

    /// Directory for log storage
    #[arg(
        short = 'l',
        long = "log-dir",
        value_name = "LOG_PATH",
        help = "Log directory [required for file logging]"
    )]
    log_dir: Option<String>,

    /// Database file path
    #[arg(
        long = "database",
        value_name = "FILE",
        help = "SQLite database file [required for persistent storage]"
    )]
    database: Option<String>,

    /// URLs to download (direct mode only)
    #[arg(help = "URLs to download immediately (not allowed in daemon mode)")]
    urls: Vec<String>,
}

impl Cli {
    pub fn validate(&self) -> Result<(), clap::Error> {
        if self.daemon && !self.urls.is_empty() {
            return Err(clap::Error::raw(
                clap::error::ErrorKind::ArgumentConflict,
                format!(
                    "Cannot accept URLs in daemon mode. URLs provided: {}. Use RPC to add downloads after starting.",
                    self.urls.join(", ")
                ),
            ));
        }

        if !self.daemon && self.urls.is_empty() {
            return Err(clap::Error::raw(
                clap::error::ErrorKind::MissingRequiredArgument,
                "Requires at least one URL in direct mode. Add URLs or use --daemon",
            ));
        }

        Ok(())
    }
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    if let Err(e) = cli.validate() {
        e.exit();
    }
    let log_level = match cli.log_level.as_str() {
        "trace" => Level::TRACE,
        "debug" => Level::DEBUG,
        "info" => Level::INFO,
        "warn" => Level::WARN,
        "error" => Level::ERROR,
        _ => {
            eprintln!("Invalid log level specified. Defaulting to INFO");
            Level::INFO
        }
    };

    match init_logger(LogConfig {
        component: Component::Vayuget,
        log_dir: cli.log_dir.clone(),
        max_level: log_level,
        log_to_console: true,
        env_filter: None,
        silent_deps: get_engine_silent_deps(),
    }) {
        Ok(_) => tracing::info!("Logger initialized successfully"),
        Err(e) => {
            eprintln!("Failed to initialize logger: {}", e);
            std::process::exit(1);
        }
    };

    fs::create_dir_all("data").expect("Failed to create data directory");
    let db_path = "data/downloads.db";
    let pool = SqlitePool::connect(&format!("sqlite:{db_path}?mode=rwc"))
        .await
        .expect("Failed to connect to the database");

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS downloads (
            id INTEGER PRIMARY KEY NOT NULL,
            request TEXT NOT NULL,
            time_stamps TEXT NOT NULL,
            config TEXT NOT NULL,
            chunks_info TEXT NOT NULL
        )",
    )
    .execute(&pool)
    .await
    .expect("Failed to create downloads table");

    let shutdown_tx = ctrl_c::ctrl_c();
    let (sender, receiver) = mpsc::channel::<ManagerCommand>(10);
    let mut download_manager = DownloadManager::new(
        3,
        sender.clone(),
        cli.daemon,
        cli.pretty_print.unwrap_or(!cli.daemon),
        pool,
    )
    .await;
    if cli.daemon {
        tracing::info!("Starting vayu rpc server");
        download_manager
            .start_server("".into(), sender.clone())
            .await;
    } else {
        tracing::info!("Starting direct download of {} URLs", cli.urls.len());
        for url in cli.urls {
            ManagerCommand::fire_forget(
                RpcRequest::DownloadRequest(DownloadRequestMessage {
                    request: DownloadRequest {
                        url,
                        directory: cli.dir.clone(),
                        rename: None,
                        headers: Headers::none(),
                    },
                    config: None,
                    info: None,
                }),
                &sender,
            )
            .await
        }
    };
    download_manager.run(receiver, sender, shutdown_tx).await;

    tracing::info!("Vayu operation completed successfully");
}
