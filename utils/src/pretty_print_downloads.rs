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

const CORNER_TOP_LEFT: &str = "┌";
const CORNER_TOP_RIGHT: &str = "┐";
const CORNER_BOTTOM_LEFT: &str = "└";
const CORNER_BOTTOM_RIGHT: &str = "┘";
const CORNER_TOP_LEFT_ROUND: &str = "╭";
const CORNER_TOP_RIGHT_ROUND: &str = "╮";
const CORNER_BOTTOM_LEFT_ROUND: &str = "╰";
const CORNER_BOTTOM_RIGHT_ROUND: &str = "╯";

const BORDER_HORIZONTAL: &str = "─";
const BORDER_VERTICAL: &str = "│";
const PADDING_X: &str = " ";

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
    let border = true;
    let rounded = false;
    let download_height = if border { 5 } else { 4 };

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

        let border_left = if border {
            format!("{BORDER_VERTICAL}{PADDING_X}")
        } else {
            String::new()
        }
        .bright_black();
        let border_right = if border {
            format!("{PADDING_X}{BORDER_VERTICAL}")
        } else {
            String::new()
        }
        .bright_black();

        let border_top = if border {
            format!(
                "{TAB_SPACE}{}{}{}",
                if rounded {
                    CORNER_TOP_LEFT_ROUND
                } else {
                    CORNER_TOP_LEFT
                },
                BORDER_HORIZONTAL.repeat(progress_bar_width + 2),
                if rounded {
                    CORNER_TOP_RIGHT_ROUND
                } else {
                    CORNER_TOP_RIGHT
                }
            )
        } else {
            String::new()
        }
        .bright_black();

        let border_bottom = if border {
            format!(
                "{TAB_SPACE}{}{}{}",
                if rounded {
                    CORNER_BOTTOM_LEFT_ROUND
                } else {
                    CORNER_BOTTOM_LEFT
                },
                BORDER_HORIZONTAL.repeat(progress_bar_width + 2),
                if rounded {
                    CORNER_BOTTOM_RIGHT_ROUND
                } else {
                    CORNER_BOTTOM_RIGHT
                }
            )
        } else {
            String::new()
        }
        .bright_black();

        let stats = format!(
            "{}{}/{}({}) Parts:{} Speed:{} Time:{}{}",
            if border { "" } else { "[" },
            downloaded,
            total,
            percentage.blue(),
            parts,
            current_speed,
            time,
            if border { "" } else { "]" }
        );

        // clear the line, go to next line, clear the line, add a tab then do the business
        println!(
            "{CLEAR_LINE}{}\n{CLEAR_LINE}{TAB_SPACE}{}{}. {} {}{}{}",
            border_top,
            border_left,
            index + 1,
            filename,
            " ".repeat(
                progress_bar_width - (4 + filename.chars().count() + status.chars().count())
            ),
            status,
            border_right
        );
        println!(
            "{CLEAR_LINE}{TAB_SPACE}{}{}{}{}",
            border_left,
            stats,
            " ".repeat({
                let len = visible_length(&stats);
                if len <= progress_bar_width {
                    progress_bar_width - len
                } else {
                    0
                }
            }),
            border_right,
        );
        println!(
            "{TAB_SPACE}{}{}{}",
            border_left,
            get_progress_string(
                download.progress_percentage().unwrap_or(0.),
                progress_bar_width,
            ),
            border_right
        );
        if border {
            println!("{}", border_bottom);
        }
    }

    println!("{CLEAR_LINE}");
    if clear_after_print {
        print!(
            "{}",
            format!("{MOVE_UP}").repeat((downloads.len() * download_height) + 2)
        );
    }
}

fn get_progress_string(progress: f64, width: usize) -> String {
    let progress = if progress == 100.0 {
        100.0
    } else {
        progress % (100 as f64)
    };
    let green_bars = ((width as f64) * (progress / (100 as f64))).round() as usize;
    format!(
        "{}{}",
        "━".repeat(green_bars).green(),
        "━".repeat(width - green_bars).bright_black(),
    )
}

fn visible_length(s: &str) -> usize {
    let mut in_escape = false;
    let mut count = 0;

    for c in s.chars() {
        if in_escape {
            // Inside escape sequence - look for termination
            if c == 'm' {
                in_escape = false;
            }
        } else if c == '\x1b' {
            // Start of escape sequence
            in_escape = true;
        } else {
            // Regular visible character
            count += 1;
        }
    }

    count
}
