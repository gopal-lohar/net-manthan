use download_engine::download_config::DownloadConfig;
use tokio;
use tracing::{debug, info};
use utils::logging::{self, Component, LogConfig};

#[tokio::main]
async fn main() {
    // Initialize logging
    match logging::init_logging(LogConfig {
        component: Component::NetManthan,
        log_dir: ".dev/logs".into(),
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
    info!("Net Manthan Finished.");
}
