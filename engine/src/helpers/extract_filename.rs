use reqwest::header::{CONTENT_DISPOSITION, HeaderMap};
use tracing::trace;
use url::Url;

/// Extract filename from response headers and URL
///
/// This function tries multiple approaches to get the filename:
/// 1. From Content-Disposition header
/// 2. From the URL path
pub fn extract_filename(headers: &HeaderMap, url: &str) -> Option<String> {
    // Try to get filename from Content-Disposition header
    if let Some(filename) = extract_filename_from_headers(headers) {
        trace!("got a file name from response headers: {}", filename);
        return Some(filename);
    }
    // Try to get filename from URL
    if let Some(filename) = extract_filename_from_url(url) {
        trace!("got a file name from url: {}", filename);
        return Some(filename);
    }

    None
}

/// Extract filename from URL
fn extract_filename_from_url(url_str: &str) -> Option<String> {
    let url = Url::parse(url_str).ok()?;
    let path = url.path();
    if path.ends_with('/') {
        return None;
    }
    let filename = path.split('/').filter(|s| !s.is_empty()).next_back()?;
    (!filename.is_empty()).then_some(sanitize_filename(filename))
}

/// Extract filename from HTTP headers from Reqwest crate
fn extract_filename_from_headers(headers: &HeaderMap) -> Option<String> {
    let content_disposition = headers.get(CONTENT_DISPOSITION)?;
    let content_disposition = content_disposition.to_str().ok()?;

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
            if let Some(end_pos) = filename.find([';', ' ']) {
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
            let end_pos = value.find([';', ' ']).unwrap_or(value.len());
            let filename = &value[..end_pos];
            if !filename.is_empty() {
                return Some(sanitize_filename(filename));
            }
        }
    }

    None
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
