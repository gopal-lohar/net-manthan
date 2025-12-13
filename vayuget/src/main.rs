use clap::Parser;
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
    config::Config,
    logger::{Component, LogConfig, get_engine_silent_deps, init_logger},
    rpc::{messages::RpcRequest, server::ManagerCommand},
};

pub mod ctrl_c;
pub mod download_manager;

#[derive(Parser)]
#[command(
    author,
    version,
    name = "vayuget",
    about = "A high-performance download manager backend"
)]
pub struct Cli {
    #[clap(flatten)]
    config: Config,
    /// URLs to download
    #[arg(group = "input")]
    urls: Vec<String>,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let config = cli.config;
    let log_level = match config.vayuget.log_level.as_str() {
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
        log_dir: Some(config.vayuget.log_path.to_str().unwrap().to_string()),
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

    fs::create_dir_all(&config.vayuget.db_path.parent().unwrap())
        .expect("Failed to create data directory");
    let pool = SqlitePool::connect(&format!(
        "sqlite:{}?mode=rwc",
        config.vayuget.db_path.to_str().unwrap()
    ))
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
    let is_daemon = cli.urls.is_empty();
    let mut download_manager = DownloadManager::new(
        3,
        sender.clone(),
        is_daemon,
        config.vayuget.pretty_print.unwrap_or(!is_daemon),
        pool,
        config.rpc.clone(),
    )
    .await;
    let is_daemon = cli.urls.is_empty();
    if is_daemon {
        tracing::info!("Starting vayu rpc server");
        download_manager
            .start_server(config.rpc.native_rpc_settings.rpc_secret, sender.clone())
            .await;
    } else {
        tracing::info!("Starting direct download of {} URLs", cli.urls.len());
        for url in cli.urls {
            ManagerCommand::fire_forget(
                RpcRequest::DownloadRequest(DownloadRequestMessage {
                    request: DownloadRequest {
                        url,
                        directory: config
                            .vayuget
                            .downloads_path
                            .to_str()
                            .unwrap()
                            .to_string(),
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
