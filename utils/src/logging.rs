use std::path::PathBuf;
use tracing::info;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

// TODO: make envFilter dynamic and remove the default log, instead we'll do it in where we use
/// Initialize a basic logging system with console and file output
pub fn init_logger(app_name: &str, log_dir: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    std::fs::create_dir_all(&log_dir)?;

    // Set up a file appender with daily rotation
    let file_appender =
        RollingFileAppender::new(Rotation::DAILY, log_dir, format!("{}.log", app_name));

    // Create a non-blocking writer
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    // Important: we need to store this guard to keep the logger working
    // Using a global static or Box::leak
    Box::leak(Box::new(_guard));

    // Create a layer for terminal output
    let terminal_layer = fmt::layer().with_ansi(true).with_target(true);

    // Create a layer for file output
    let file_layer = fmt::layer()
        .with_ansi(false)
        .with_target(true)
        .with_writer(non_blocking);

    // Create a filter based on RUST_LOG env var or default to info
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        EnvFilter::new(format!(
            "{}=info,ui=info,net_manthan=info",
            env!("CARGO_PKG_NAME")
        ))
    });

    // Install both layers with a single init call
    tracing_subscriber::registry()
        .with(filter)
        .with(terminal_layer)
        .with(file_layer)
        .init();

    info!("Logging initialized");

    Ok(())
}
