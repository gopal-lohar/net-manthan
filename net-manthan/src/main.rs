use std::time::Duration;

use tokio;
use tracing::info;
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

    tokio::time::sleep(Duration::from_secs(10)).await;

    info!("Net Manthan Finished.");
}
