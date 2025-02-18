use crate::{Cli, Commands};
use color_eyre::Result;
use tracing::info;

pub fn run(cli: Cli) -> Result<()> {
    info!("Running CLI");

    // Handle subcommands
    match &cli.command {
        Commands::List {
            incomplete,
            detailed,
            limit,
        } => {
            info!("Listing downloads");
            info!("Incomplete: {}", incomplete);
            info!("Detailed: {}", detailed);
            info!("Limit: {:?}", limit);
        }
        Commands::Start {
            url,
            output,
            threads,
        } => {
            info!("Starting download with URL: {}", url);
            info!("Output path: {:?}", output);
            info!("Threads: {:?}", threads);
        }
        Commands::Resume { ids } => {
            info!("Resuming downloads: {:?}", ids);
        }
        Commands::Pause { ids } => {
            info!("Pausing downloads: {:?}", ids);
        }
        Commands::Watch {
            ids,
            interval,
            detailed,
        } => {
            info!("Watching downloads: {:?}", ids);
            info!("Interval: {}", interval);
            info!("Detailed: {}", detailed);
        }
        Commands::Remove { ids, delete_files } => {
            info!("Removing downloads: {:?}", ids);
            info!("Delete files: {}", delete_files);
        }
        Commands::Update { id, url, output } => {
            info!("Updating download: {}", id);
            info!("New URL: {:?}", url);
            info!("New output: {:?}", output);
        }
        Commands::Config { action } => match action {
            crate::ConfigCommands::Set {
                auto_resume,
                threads,
                single_threaded_buffer_size_in_kb,
                multi_threaded_buffer_size_in_kb,
                download_dir,
                database,
            } => {
                info!("Setting config");
                info!("Auto resume: {:?}", auto_resume);
                info!("Threads: {:?}", threads);
                info!(
                    "Single threaded buffer size: {:?}",
                    single_threaded_buffer_size_in_kb
                );
                info!(
                    "Multi threaded buffer size: {:?}",
                    multi_threaded_buffer_size_in_kb
                );
                info!("Download dir: {:?}", download_dir);
                info!("Database: {:?}", database);
            }
            crate::ConfigCommands::Show => {
                info!("Getting config");
            }
        },
    }

    Ok(())
}
