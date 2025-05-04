use std::path::PathBuf;
use tokio;
use tracing::info;
use utils::logging;

#[tokio::main]
async fn main() {
    match logging::init_logger("Net Manthan", PathBuf::from(".dev")) {
        Ok(_) => (),
        Err(e) => {
            eprintln!("Failed to initialize logger: {}", e);
        }
    }

    info!("Starting Net Manthan");
}
