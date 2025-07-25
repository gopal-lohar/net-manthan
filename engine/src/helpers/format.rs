use chrono::{DateTime, Duration, Local, Utc};

// Helper function to format bytes
pub fn format_bytes(bytes: u64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;

    match bytes {
        s if s < KB as u64 => format!("{s}B"),
        s if s < MB as u64 => format!("{:.2}KB", s as f64 / KB),
        s if s < GB as u64 => format!("{:.2}MB", s as f64 / MB),
        _ => format!("{:.2}GB", bytes as f64 / GB),
    }
}

// Helper function to format speed
pub fn format_speed(speed: u64) -> String {
    format!("{}/s", format_bytes(speed))
}

// Helper function to format duration
pub fn format_duration(duration: Duration) -> String {
    let total_secs = duration.num_seconds();
    if total_secs == 0 {
        return "<1s".to_string();
    }

    let days = total_secs / 86_400;
    let hours = (total_secs % 86_400) / 3600;
    let minutes = (total_secs % 3600) / 60;
    let seconds = total_secs % 60;

    let mut parts = Vec::new();

    if days > 0 {
        parts.push(format!("{days}d"));
    }
    if hours > 0 {
        parts.push(format!("{hours}h"));
    }
    if minutes > 0 && parts.len() < 2 {
        parts.push(format!("{minutes}m"));
    }
    if seconds > 0 && parts.len() < 2 {
        parts.push(format!("{seconds}s"));
    }

    if parts.is_empty() {
        "<1s".to_string()
    } else {
        parts.join(" ")
    }
}

pub fn format_utc_to_local(utc: DateTime<Utc>) -> String {
    // utc.with_timezone(&Local).format("%Y-%m-%d %H:%M:%S").to_string()
    utc.with_timezone(&Local)
        .format("%b %d, %Y %I:%M %p")
        .to_string()
}
