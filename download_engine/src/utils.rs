use reqwest::header::{CONTENT_DISPOSITION, HeaderMap};
use std::path::PathBuf;
use url::Url;

/// format speed from bytes per second
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

/// split total size into chunks with ~equal size
pub fn calculate_chunks(total_size: u64, num_chunks: u64) -> Vec<(u64, u64)> {
    let chunk_size = total_size / num_chunks;
    let remainder = total_size % num_chunks;
    (0..num_chunks)
        .filter_map(|i| {
            let start = i * chunk_size + u64::min(i, remainder);
            let end = if i < remainder {
                start + chunk_size // chunk_size+1 when remainder > 0
            } else {
                start + chunk_size - 1
            };
            (end < total_size).then(|| (start, end))
        })
        .collect()
}

/// Extract filename from response headers and URL
///
/// This function tries multiple approaches to get the filename:
/// 1. From Content-Disposition header
/// 2. From the URL path
pub fn extract_filename(headers: &HeaderMap, url: &str) -> Option<PathBuf> {
    // Try to get filename from Content-Disposition header
    if let Some(filename) = extract_filename_from_content_disposition(headers) {
        return Some(PathBuf::from(filename));
    }

    // Try to get filename from URL
    if let Some(filename) = extract_filename_from_url(url) {
        return Some(PathBuf::from(filename));
    }

    None
}

/// Extract filename from Content-Disposition header
fn extract_filename_from_content_disposition(headers: &HeaderMap) -> Option<String> {
    let content_disposition = headers.get(CONTENT_DISPOSITION)?;
    let content_disposition = content_disposition.to_str().ok()?;

    // Handle various Content-Disposition formats

    // 1. Try filename=
    if let Some(pos) = content_disposition.find("filename=") {
        let start = pos + "filename=".len();
        let mut filename = content_disposition[start..].trim_start();

        // Handle quoted filenames
        if filename.starts_with('"') && filename.len() > 1 {
            filename = &filename[1..];
            if let Some(end_quote) = filename.find('"') {
                filename = &filename[..end_quote];
            }
        } else {
            // Non-quoted filename ends at first semicolon or whitespace
            if let Some(end_pos) = filename.find(|c| c == ';' || c == ' ') {
                filename = &filename[..end_pos];
            }
        }

        if !filename.is_empty() {
            return Some(sanitize_filename(filename));
        }
    }

    // 2. Try filename*=
    if let Some(pos) = content_disposition.find("filename*=") {
        let start = pos + "filename*=".len();
        let value = content_disposition[start..].trim_start();

        // Handle UTF-8 encoding format: UTF-8''filename
        if value.starts_with("UTF-8''") || value.starts_with("utf-8''") {
            let encoded_filename = value.split('\'').nth(2)?;
            let decoded = percent_decode(encoded_filename);
            if !decoded.is_empty() {
                return Some(sanitize_filename(&decoded));
            }
        } else {
            // Simple case without encoding specification
            let end_pos = value.find(|c| c == ';' || c == ' ').unwrap_or(value.len());
            let filename = &value[..end_pos];
            if !filename.is_empty() {
                return Some(sanitize_filename(filename));
            }
        }
    }

    None
}

/// Extract filename from URL path
fn extract_filename_from_url(url_str: &str) -> Option<String> {
    let url = Url::parse(url_str).ok()?;
    let path = url.path();

    // Get the last segment of the path
    let segments: Vec<&str> = path.split('/').collect();
    let last_segment = segments.last()?;

    if last_segment.is_empty() {
        None
    } else {
        // Remove query parameters if present
        let clean_segment = last_segment.split('?').next().unwrap_or(last_segment);
        Some(percent_decode(&sanitize_filename(clean_segment)))
    }
}

/// URL-decode a percent-encoded string
fn percent_decode(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let mut bytes = input.bytes().peekable();

    while let Some(byte) = bytes.next() {
        if byte == b'%' {
            let hex1 = bytes.next();
            let hex2 = bytes.next();

            if let (Some(h1), Some(h2)) = (hex1, hex2) {
                if let (Some(d1), Some(d2)) = (decode_hex_digit(h1), decode_hex_digit(h2)) {
                    let decoded_byte = (d1 << 4) | d2;
                    // Add as UTF-8 character
                    output.push(decoded_byte as char);
                    continue;
                }
            }

            // If we can't decode, just add the percent sign and continue
            output.push('%');
            if let Some(h1) = hex1 {
                output.push(h1 as char);
            }
            if let Some(h2) = hex2 {
                output.push(h2 as char);
            }
        } else if byte == b'+' {
            // In some encodings, + represents space
            output.push(' ');
        } else {
            output.push(byte as char);
        }
    }

    output
}

/// Convert a hex character to its decimal value
fn decode_hex_digit(digit: u8) -> Option<u8> {
    match digit {
        b'0'..=b'9' => Some(digit - b'0'),
        b'A'..=b'F' => Some(digit - b'A' + 10),
        b'a'..=b'f' => Some(digit - b'a' + 10),
        _ => None,
    }
}

/// Sanitize filename by removing invalid characters
fn sanitize_filename(filename: &str) -> String {
    // List of characters not allowed in filenames on most platforms
    const INVALID_CHARS: &[char] = &['/', '\\', ':', '*', '?', '"', '<', '>', '|'];

    let sanitized: String = filename
        .chars()
        .map(|c| if INVALID_CHARS.contains(&c) { '_' } else { c })
        .collect();

    // Trim leading/trailing whitespace and dots
    let sanitized = sanitized.trim().trim_start_matches('.');

    if sanitized.is_empty() {
        String::from("unnamed_file")
    } else {
        sanitized.to_string()
    }
}
