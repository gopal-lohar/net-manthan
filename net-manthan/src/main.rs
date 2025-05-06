use download_engine::{Download, download_config::DownloadConfig, types::DownloadRequest};
use tokio;
use tracing::{debug, error, info};
use utils::logging::{self, Component, LogConfig};

#[tokio::main]
async fn main() {
    // Initialize logging
    match logging::init_logging(LogConfig {
        component: Component::NetManthan,
        log_dir: ".dev/logs".into(),
        silent_deps: vec!["hyper_util".into(), "mio".into()],
        ..Default::default()
    }) {
        Ok(_) => {
            info!("Logger initialized for {}", Component::NetManthan.as_str());
        }
        Err(e) => {
            eprintln!("Failed to initialize logger: {}", e);
        }
    }

    let download_config = DownloadConfig::default();
    debug!("download_config = {:?}", download_config);

    let mut download_windows = Download::new(DownloadRequest {
        url: "https://download.microsoft.com/download/6/8/3/683178b7-baac-4b0d-95be-065a945aadee/Windows11InstallationAssistant.exe".into(),
        file_dir: "/tmp/".into(),
        file_name: None,
        referrer: None,
        headers: None,
    }, &download_config);
    debug!("download = {:?}", download_windows);

    match download_windows.load_download_info().await {
        Ok(_) => {
            info!("Download info loaded successfully");
        }
        Err(e) => {
            error!("Failed to load download info: {}", e);
        }
    }

    let mut download_docker = Download::new(
        DownloadRequest {
            url: "https://desktop.docker.com/win/main/amd64/Docker%20Desktop%20Installer.exe"
                .to_string(),
            file_dir: "/tmp/".into(),
            file_name: None,
            referrer: None,
            headers: None,
        },
        &download_config,
    );

    match download_docker.load_download_info().await {
        Ok(_) => {
            info!("Download info loaded successfully");
        }
        Err(e) => {
            error!("Failed to load download info: {}", e);
        }
    }

    debug!("download_docker = {:?}", download_docker);

    info!("Net Manthan Finished.");
}
