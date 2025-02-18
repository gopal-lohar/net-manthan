use crate::run::run;
use clap::{Parser, Subcommand};
use color_eyre::Result;
use std::path::PathBuf;
use tracing::info;
use tracing_subscriber::{self, EnvFilter};

mod run;

/// Advanced Download Manager CLI
#[derive(Parser)]
#[command(author, version, about)]
pub struct Cli {
    #[arg(short, long, action = clap::ArgAction::Count)]
    debug: u8,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// List downloads with optional filters
    List {
        /// Show only incomplete downloads
        #[arg(short, long)]
        incomplete: bool,

        /// Show detailed information
        #[arg(short, long)]
        detailed: bool,

        /// Number of recent downloads to show
        #[arg(short, long)]
        limit: Option<usize>,
    },

    /// Start a new download
    Start {
        /// Download URL
        url: String,

        /// Custom output path (optional)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Number of threads (default from config)
        #[arg(short, long)]
        threads: Option<u8>,
    },

    /// Resume paused downloads
    Resume {
        /// Download IDs to resume (if none, resumes all)
        ids: Vec<u64>,
    },

    /// Pause active downloads
    Pause {
        /// Download IDs to pause (if none, pauses all)
        ids: Vec<u64>,
    },

    /// Show real-time progress of downloads
    Watch {
        /// Download IDs to watch (if none, watches all)
        ids: Vec<u64>,

        /// Update interval in milliseconds
        #[arg(short, long, default_value = "500")]
        interval: u64,

        /// Show detailed progress information
        #[arg(short, long)]
        detailed: bool,
    },

    /// Remove downloads
    Remove {
        /// Download IDs to remove
        ids: Vec<u64>,

        /// Also delete downloaded files
        #[arg(short, long)]
        delete_files: bool,
    },

    /// Update download properties
    Update {
        /// Download ID to update
        id: u64,

        /// New URL for the download
        #[arg(short, long)]
        url: Option<String>,

        /// New output path
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Manage configuration
    Config {
        #[command(subcommand)]
        action: ConfigCommands,
    },
}

#[derive(Subcommand)]
pub enum ConfigCommands {
    /// Show current configuration
    Show,

    /// Set configuration values
    Set {
        /// Auto-resume downloads on startup
        #[arg(long)]
        auto_resume: Option<bool>,

        /// Default number of threads
        #[arg(long)]
        threads: Option<u8>,

        /// buffer size in single threaded download
        #[arg(long)]
        single_threaded_buffer_size_in_kb: u64,

        /// Buffer size (per thread) in multi-threaded download
        #[arg(long)]
        multi_threaded_buffer_size_in_kb: u64,

        /// Default download directory
        #[arg(long)]
        download_dir: Option<PathBuf>,

        /// Database path
        #[arg(long)]
        database: Option<PathBuf>,
    },
}

fn setup_logging(verbosity: u8) {
    let filter = match verbosity {
        0 => "error", // Default: only errors
        1 => "warn",  // -v: warnings too
        2 => "info",  // -vv: info too
        _ => "trace", // -vvv: everything
    };

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new(filter))
        .init();
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let cli = Cli::parse();
    setup_logging(cli.debug);
    info!("CLI parsed");
    run(cli)?;
    Ok(())
}

// Example usage messages (would be shown in --help):
/*
USAGE:
    dm list --incomplete            # Show incomplete downloads
    dm list --limit 10             # Show 10 most recent downloads
    dm start <URL> -o path         # Start new download with custom path
    dm watch                       # Watch all active downloads
    dm watch 1 2 3                 # Watch specific downloads
    dm pause                       # Pause all downloads
    dm resume 1                    # Resume download #1
    dm remove 1 2 --delete-files   # Remove downloads and their files
    dm update 1 --url "new-url"    # Update download #1's URL
    dm config set --threads 4      # Set default thread count
*/
