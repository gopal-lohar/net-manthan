use tokio;
use tracing::info;
use utils::logging::{self, Component, LogConfig};

#[tokio::main]
async fn main() {
    // Initialize logging
    match logging::init_logging(LogConfig {
        component: Component::NetManthan,
        log_dir: ".dev/logs".into(),
        silent_deps: vec!["naga".to_string(), "blade_graphics".to_string()],
        ..Default::default()
    }) {
        Ok(_) => {
            info!("Logger initialized for {}", Component::Ui.as_str());
        }
        Err(e) => {
            eprintln!("Failed to initialize logger: {}", e);
        }
    }

    info!("Starting Net Manthan");
}
