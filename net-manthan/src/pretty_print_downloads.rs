use std::path::PathBuf;

use colored::Colorize;
use download_engine::{
    Download, DownloadParts,
    types::DownloadStatus,
    utils::{format_bytes, format_duration},
};

const TAB_SPACE: &str = "  ";
const CLEAR_LINE: &str = "\x1B[K";

/// Prints the progress of a vector of downloads in pretty format in terminal
pub fn pretty_print_downloads(downloads: &mut Vec<Download>, clear_after_print: bool) {
    // there are 4 goals here
    // 1. printing the downloads in a pretty way spaced 2 lines away from the top
    // 2. the logs should keep flowing from the top while the downloads keep updating
    // 3. when pressing ctrl+c the download should stay there (not overwritten by system prompt)
    // 4. the filename and status should be aligned in both ends of progress bar and the width
    // of the progress bar should be customizable
    //
    // 1. all these lines in download are printed after clearing the line
    // in case any of following doesn't go right eg. the buffer flushes
    // 2. we print the progress and the go back to top but not using println, using print!() (conditonally)
    // this way the cosole doesn't get updated until some \n flows into the buffer
    // either through logs or through the beginning print statement.
    // 3. since sending the cursor to top wasn't flushed so cursor is still at bottom
    // hence default prompt prints after the cursor and doesn't overwrite anything
    // 3. calcualtoins are made on the basis of progress_bar_width variable

    let progress_bar_width = 75;
    let max_filename_len = progress_bar_width - 15;

    for (index, download) in &mut downloads.iter_mut().enumerate() {
        let mut filename = download
            .file_name
            .clone()
            .unwrap_or(PathBuf::from("unnamed"))
            .to_string_lossy()
            .into_owned();
        filename = if filename.len() > max_filename_len {
            format!("{}...", &filename[..max_filename_len - 3])
        } else {
            filename
        };
        let status = match download.get_status() {
            DownloadStatus::Downloading => "Downloading".blue(),
            DownloadStatus::Complete => "Complete".green(),
            DownloadStatus::Failed => "Failed".red(),
            DownloadStatus::Cancelled => "Cancelled".red(),
            _ => format!("{:?}", download.get_status()).red(),
        };

        let downloaded = format_bytes(download.get_bytes_downloaded());
        let total = format_bytes(download.get_total_size());
        let percentage = format!("{}%", download.get_progress_percentage() as usize,);
        let parts = match &download.parts {
            DownloadParts::NonResumable(_) => 1,
            DownloadParts::Resumable(p) => p.len(),
            DownloadParts::None => 0,
        }
        .to_string();
        let current_speed = match download.get_status() {
            DownloadStatus::Downloading => {
                format!("{}", download.get_formatted_current_speed()).green()
            }
            _ => format!("{}", download.get_formatted_average_speed()).normal(),
        };
        let eta = if matches!(download.get_status(), DownloadStatus::Complete) {
            "".into()
        } else if download.get_current_speed() == 0 {
            "∞".to_string()
        } else {
            format_duration(
                (download.get_total_size() - download.get_bytes_downloaded())
                    / (download.get_current_speed() as u64),
            )
        };
        let time_elapsed = if download.active_time.as_seconds_f64() < 0. {
            format_duration(0)
        } else {
            format_duration(download.active_time.as_seconds_f64() as u64)
        };

        let time = if matches!(download.get_status(), DownloadStatus::Complete) {
            time_elapsed.yellow()
        } else {
            format!("{}/{}", time_elapsed.yellow(), eta.yellow()).normal()
        };

        // clear the line, go to next line, clear the line, add a tab then do the business
        println!(
            "{CLEAR_LINE}\n{CLEAR_LINE}{TAB_SPACE}{}. {} {}{}",
            index + 1,
            filename,
            " ".repeat(
                progress_bar_width - (4 + filename.chars().count() + status.chars().count())
            ),
            status,
        );
        println!(
            "{CLEAR_LINE}{TAB_SPACE}[{}/{}({}) Parts:{} Speed:{} Time:{}]",
            downloaded,
            total,
            percentage.blue(),
            parts,
            current_speed,
            time
        );
        print_progress_string(download.get_progress_percentage(), progress_bar_width);
    }

    println!("{CLEAR_LINE}");
    if clear_after_print {
        print!("\x1B[{}A", (downloads.len() * 4) + 1);
    }
}

fn print_progress_string(progress: f64, width: usize) {
    let progress = if progress == 100.0 {
        100.0
    } else {
        progress % (100 as f64)
    };
    let green_bars = ((width as f64) * (progress / (100 as f64))).round() as usize;
    println!(
        "{TAB_SPACE}{}{}",
        "━".repeat(green_bars).green(),
        "━".repeat(width - green_bars).bright_black()
    )
}
