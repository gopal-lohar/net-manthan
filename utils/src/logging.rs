use std::path::PathBuf;
use std::sync::Once;
use tracing::Level;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{EnvFilter, Layer, fmt, layer::SubscriberExt, util::SubscriberInitExt};

// Global initialization guard
static INIT: Once = Once::new();

/// Application component identifier
pub enum Component {
    Ui,
    NetManthan,
}

impl Component {
    pub fn as_str(&self) -> &'static str {
        match self {
            Component::Ui => "ui",
            Component::NetManthan => "net-manthan",
        }
    }
}

/// Configuration for logging initialization
pub struct LogConfig {
    /// Component name for log identification
    pub component: Component,
    /// Directory where log files will be stored
    pub log_dir: PathBuf,
    /// Maximum log level
    pub max_level: Level,
    /// Whether to also log to stdout
    pub log_to_console: bool,
    /// Optional custom env filter string
    pub env_filter: Option<String>,
    /// List of dependency crates to silence
    pub silent_deps: Vec<String>,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            component: Component::Ui,
            log_dir: PathBuf::from("logs"),
            max_level: Level::TRACE,
            log_to_console: true,
            env_filter: None,
            silent_deps: Vec::new(),
        }
    }
}

/// Initialize logging for the application
pub fn init_logging(config: LogConfig) -> Result<(), Box<dyn std::error::Error>> {
    let mut result = Ok(());

    INIT.call_once(|| {
        result = match initialize_logging_internal(config) {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        };
    });

    result
}

fn initialize_logging_internal(config: LogConfig) -> Result<(), Box<dyn std::error::Error>> {
    // Create log directory if it doesn't exist
    std::fs::create_dir_all(&config.log_dir)?;

    // Set up file logging
    let file_appender = RollingFileAppender::new(
        Rotation::DAILY,
        &config.log_dir,
        format!("{}.log", config.component.as_str()),
    );

    // Start with a file layer
    let mut layers = Vec::new();
    let file_layer = fmt::Layer::new()
        .with_ansi(false)
        .with_writer(file_appender)
        .with_target(true);

    // Add console layer if configured
    if config.log_to_console {
        let stdout_layer = fmt::Layer::new()
            .with_ansi(true)
            .with_target(true)
            .compact();

        layers.push(stdout_layer.with_filter(build_filter(&config)?).boxed());
    }

    // Add file layer
    layers.push(file_layer.with_filter(build_filter(&config)?).boxed());

    // Initialize with all the layers
    tracing_subscriber::registry().with(layers).try_init()?;

    Ok(())
}

fn build_filter(config: &LogConfig) -> Result<EnvFilter, Box<dyn std::error::Error>> {
    let mut filter = if let Some(filter_str) = &config.env_filter {
        EnvFilter::try_new(filter_str)?
    } else {
        EnvFilter::try_new(format!("{}", config.max_level))?
            .add_directive(format!("{}={}", config.component.as_str(), config.max_level).parse()?)
    };

    // Apply silencing for noisy dependencies
    for dep in &config.silent_deps {
        filter = filter.add_directive(format!("{}=error", dep).parse()?);
    }

    Ok(filter)
}

pub fn get_ui_config(log_dir: &str) -> LogConfig {
    LogConfig {
        component: Component::Ui,
        log_dir: log_dir.into(),
        silent_deps: vec![
            "naga".to_string(),
            "blade_graphics".to_string(),
            "Users".to_string(),
            "cosmic_text".to_string(),
            "polling".to_string(),
            "mio".to_string(),
            "perform".to_string(),
            "async_io".to_string(),
            "zbus".to_string(),
            "calloop".to_string(),
            "gpui".to_string(),
        ],
        ..Default::default()
    }
}
