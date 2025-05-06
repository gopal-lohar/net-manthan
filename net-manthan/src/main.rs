use download_engine::{Download, download_config::DownloadConfig, types::DownloadRequest};
use tokio;
use tracing::{debug, info, trace};
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

    let download = Download::new(DownloadRequest {
        url: "https://example.com/file.zip".to_string(),
        file_dir: "/path/to/download".into(),
        file_name: None,
        referrer: None,
        headers: None,
    });
    debug!("download = {:?}", download);
    debug!("total_size = {}", download.get_total_size());
    trace!("status = {:?}", download.get_status());
    info!("Net Manthan Finished.");
}
