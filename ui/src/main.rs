use std::sync::Arc;

use download_manager::DownloadManager;
use iced::{Pixels, Task, application, window};
use styles::constants::FONT_SIZE_BODY;
use tracing::Level;
use types::config::Config;
use utils::{
    logger::{Component, LogConfig, get_ui_silent_deps, init_logger},
    rpc::{NativeRpcSettings, RpcConfig, client::Client},
};

pub mod components;
pub mod download_manager;
pub mod styles;
pub mod types;

pub const DOWNLOAD_MANAGER_TITLE: &str = "Vayuget";
pub const WINDOW_ID: &str = "Vayuget";

#[tokio::main]
async fn main() -> iced::Result {
    let config = Config::default();
    match init_logger(LogConfig {
        component: Component::Ui,
        log_dir: None,
        max_level: Level::TRACE,
        log_to_console: true,
        env_filter: None,
        silent_deps: get_ui_silent_deps(),
    }) {
        Ok(_) => tracing::info!("Logger initialized successfully"),
        Err(e) => {
            eprintln!("Failed to initialize logger: {}", e);
            std::process::exit(1);
        }
    };

    let client = Client::new(
        "".into(),
        RpcConfig {
            native_rpc_settings: NativeRpcSettings {
                address: "/tmp/vayu.sock".into(),
                allow_all_users: true,
            },
        },
    );
    let _ = client.connect().await;
    let client = Arc::new(client);

    application(
        DOWNLOAD_MANAGER_TITLE,
        DownloadManager::update,
        DownloadManager::view,
    )
    .settings(iced::Settings {
        id: Some(WINDOW_ID.into()),
        antialiasing: true,
        default_text_size: Pixels(FONT_SIZE_BODY),
        ..Default::default()
    })
    .window(window::Settings {
        decorations: !config.ui.custom_decoration,
        ..Default::default()
    })
    .scale_factor(DownloadManager::scale_factor)
    .theme(DownloadManager::theme)
    .subscription(DownloadManager::subscription)
    .run_with(move || {
        (
            DownloadManager::new(client),
            Task::done(types::message::Message::Refetch),
        )
    })
}
