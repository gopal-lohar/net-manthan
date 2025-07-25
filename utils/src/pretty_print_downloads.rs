use chrono::TimeDelta;
use colored::Colorize;
use engine::{
    helpers::format::{format_bytes, format_duration, format_speed},
    types::{
        chunks::{ChunksInfo, DownloadParts},
        download::Download,
        status::DownloadStatus,
    },
};

const TAB_SPACE: &str = "  ";
const CLEAR_LINE: &str = "\x1B[K";
const MOVE_UP: &str = "\x1B[1A";

/// Prints the progress of a vector of downloads in pretty format in terminal
pub fn pretty_print_downloads(downloads: &Vec<Download>, clear_after_print: bool) {
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

    println!("{CLEAR_LINE}");
    for (index, download) in &mut downloads.iter().enumerate() {
        let mut filename = download.get_filename().unwrap_or("Unknown".into());
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

        let downloaded = format_bytes(download.bytes_downloaded());
        let total = download
            .total_size()
            .map(|size| format_bytes(size))
            .unwrap_or("Unknown".into());
        let percentage = format!(
            "{}%",
            download
                .progress_percentage()
                .map(|p| p as usize)
                .unwrap_or(0)
        );
        let parts = match &download.chunks_info {
            ChunksInfo::InfoLoaded(info) => match &info.parts {
                DownloadParts::Resumable(parts) => parts.len(),
                _ => 1,
            },
            _ => 0,
        }
        .to_string();
        let current_speed = match download.get_status() {
            DownloadStatus::Downloading => {
                format!("{}", format_speed(download.total_speed())).green()
            }
            _ => format!("{}", format_speed(download.average_speed())).normal(),
        };
        let eta = if matches!(download.get_status(), DownloadStatus::Complete) {
            "".into()
        } else {
            match download.estimated_time_remaining() {
                Some(duration) => format_duration(duration),
                None => "∞".to_string(),
            }
        };
        let time_elapsed = if download.time_stamps.active_time.as_seconds_f64() <= 0. {
            format_duration(TimeDelta::zero())
        } else {
            format_duration(download.time_stamps.active_time)
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
        print_progress_string(
            download.progress_percentage().unwrap_or(0.),
            progress_bar_width,
        );
    }

    println!("{CLEAR_LINE}");
    if clear_after_print {
        print!("{}", format!("{MOVE_UP}").repeat((downloads.len() * 4) + 2));
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
