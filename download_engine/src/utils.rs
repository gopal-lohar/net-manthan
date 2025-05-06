pub fn format_speed(bytes_per_second: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes_per_second < KB {
        format!("{} B/s", bytes_per_second)
    } else if bytes_per_second < MB {
        format!("{:.2} KB/s", bytes_per_second as f64 / KB as f64)
    } else if bytes_per_second < GB {
        format!("{:.2} MB/s", bytes_per_second as f64 / MB as f64)
    } else {
        format!("{:.2} GB/s", bytes_per_second as f64 / GB as f64)
    }
}
