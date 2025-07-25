use std::sync::Once;
use tracing::Level;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{EnvFilter, Layer, fmt, layer::SubscriberExt, util::SubscriberInitExt};

// Global initialization guard
static INIT: Once = Once::new();

/// Application component identifier
pub enum Component {
    Ui,
}

impl Component {
    pub fn as_str(&self) -> &'static str {
        match self {
            Component::Ui => "ui",
        }
    }
}

/// Configuration for logging initialization
pub struct LogConfig {
    /// Component name for log identification
    pub component: Component,
    /// Directory where log files will be stored (optional)
    pub log_dir: Option<String>,
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
            log_dir: Some("logs".to_string()),
            max_level: Level::TRACE,
            log_to_console: true,
            env_filter: None,
            silent_deps: Vec::new(),
        }
    }
}

impl LogConfig {
    /// Create a new LogConfig with console-only logging
    pub fn console_only(component: Component) -> Self {
        Self {
            component,
            log_dir: None,
            max_level: Level::TRACE,
            log_to_console: true,
            env_filter: None,
            silent_deps: Vec::new(),
        }
    }

    /// Create a new LogConfig with file logging to the specified directory
    pub fn with_file_logging<S: Into<String>>(component: Component, log_dir: S) -> Self {
        Self {
            component,
            log_dir: Some(log_dir.into()),
            max_level: Level::TRACE,
            log_to_console: true,
            env_filter: None,
            silent_deps: Vec::new(),
        }
    }

    /// Disable console logging
    pub fn no_console(mut self) -> Self {
        self.log_to_console = false;
        self
    }

    /// Set the maximum log level
    pub fn with_level(mut self, level: Level) -> Self {
        self.max_level = level;
        self
    }

    /// Set custom environment filter
    pub fn with_env_filter<S: Into<String>>(mut self, filter: S) -> Self {
        self.env_filter = Some(filter.into());
        self
    }

    /// Add dependencies to silence
    pub fn silence_deps(mut self, deps: Vec<String>) -> Self {
        self.silent_deps = deps;
        self
    }
}

/// Initialize logging for the application
pub fn init_logger(config: LogConfig) -> Result<(), Box<dyn std::error::Error>> {
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
    let mut layers = Vec::new();

    // Add file logging layer if directory is specified
    if let Some(log_dir) = &config.log_dir {
        // Create log directory if it doesn't exist
        std::fs::create_dir_all(log_dir)?;

        // Set up file logging
        let file_appender = RollingFileAppender::new(
            Rotation::DAILY,
            log_dir,
            format!("{}.log", config.component.as_str()),
        );

        let file_layer = fmt::Layer::new()
            .with_ansi(false)
            .with_writer(file_appender)
            .with_target(true);

        layers.push(file_layer.with_filter(build_filter(&config)?).boxed());
    }

    // Add console layer if configured
    if config.log_to_console {
        let stdout_layer = fmt::Layer::new()
            .with_ansi(true)
            .with_target(true)
            .compact();

        layers.push(stdout_layer.with_filter(build_filter(&config)?).boxed());
    }

    // Ensure at least one layer is present
    if layers.is_empty() {
        return Err("No logging layers configured. Enable either console or file logging.".into());
    }

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

// wgpu_core
pub fn get_ui_config() -> LogConfig {
    LogConfig {
        component: Component::Ui,
        log_dir: None,
        max_level: Level::TRACE,
        log_to_console: true,
        env_filter: None,
        silent_deps: get_ui_silent_deps(),
    }
}

pub fn get_ui_silent_deps() -> Vec<String> {
    vec![
        // for rfd
        "async_io".into(),
        "zbus".into(),
        // for iced
        "wgpu_core".into(),
        "calloop".into(),
        "polling".into(),
        "cosmic_text".into(),
        "iced_wgpu".into(),
        "iced_graphics".into(),
        "sctk".into(),
        "iced_winit".into(),
        "naga".into(),
        "wgpu_hal".into(),
        // were for gpui
        // "naga".to_string(),
        // "blade_graphics".to_string(),
        // "Users".to_string(),
        // "cosmic_text".to_string(),
        // "polling".to_string(),
        // "mio".to_string(),
        // "perform".to_string(),
        // "async_io".to_string(),
        // "zbus".to_string(),
        // "calloop".to_string(),
        // "gpui".to_string(),
    ]
}

pub fn get_engine_silent_deps() -> Vec<String> {
    vec!["hyper_util".into(), "reqwest".into()]
}
