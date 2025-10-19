use reqwest::header::{CONTENT_DISPOSITION, HeaderMap};
use tracing::trace;
use url::Url;
use percent_encoding::percent_decode_str;

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
    
    // Use path_segments for proper decoding and handling of path parts
    let filename_with_query = url.path_segments()? // This should return an iterator of DECODED segments
                          .last()? // Get the last segment
                          .to_string(); // Convert to String

    // Strip any potential query parameters that might be part of the last segment
    let filename = filename_with_query.split('?').next().unwrap_or(&filename_with_query);

    if filename.is_empty() || filename.ends_with('/') { // Check for empty or directory-like filename
        return None;
    }
    
    // Explicitly percent-decode here to see if it fixes the test
    let decoded_filename = percent_decode_str(filename).decode_utf8().ok()?;

    (!decoded_filename.is_empty()).then_some(sanitize_filename(&decoded_filename))
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
            let decoded = percent_decode_str(encoded_filename).decode_utf8().ok()?;
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

/// Sanitize filename by removing invalid characters
fn sanitize_filename(filename: &str) -> String {
    // List of characters not allowed in filenames on most platforms
    const INVALID_CHARS: &[char] = &['/', '\\', ':', '*', '?', '"', '<', '>', '|'];

    let sanitized: String = filename
        .chars()
        .map(|c| if INVALID_CHARS.contains(&c) { '_' } else { c })
        .collect();

    // Trim leading/trailing whitespace and dots
    let sanitized = sanitized.trim().trim_start_matches('.').to_string();

    if sanitized.is_empty() || sanitized.chars().all(|c| c == '_') {
        String::from("unnamed_file")
    } else {
        sanitized
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use reqwest::header::{HeaderMap, HeaderValue};

    #[test]
    fn test_extract_filename_from_url_simple() {
        let url = "http://example.com/file.txt";
        assert_eq!(extract_filename_from_url(url), Some("file.txt".to_string()));
    }

    #[test]
    fn test_extract_filename_from_url_with_path() {
        let url = "http://example.com/path/to/file.zip";
        assert_eq!(extract_filename_from_url(url), Some("file.zip".to_string()));
    }

    #[test]
    fn test_extract_filename_from_url_no_filename() {
        let url = "http://example.com/path/";
        assert_eq!(extract_filename_from_url(url), None);
    }

    #[test]
    fn test_extract_filename_from_url_with_query() {
        let url = "http://example.com/file.txt?key=value";
        assert_eq!(extract_filename_from_url(url), Some("file.txt".to_string()));
    }

    #[test]
    fn test_extract_filename_from_url_encoded_chars() {
        let url = "http://example.com/file%20with%20spaces.txt";
        assert_eq!(extract_filename_from_url(url), Some("file with spaces.txt".to_string()));
    }

    #[test]
    fn test_extract_filename_from_url_encoded_chars_complex() {
        let url = "http://example.com/file%21%40%23%24.txt";
        assert_eq!(extract_filename_from_url(url), Some("file!@#$.txt".to_string()));
    }

    #[test]
    fn test_extract_filename_from_headers_filename_only() {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_DISPOSITION, HeaderValue::from_static("attachment; filename=test.txt"));
        assert_eq!(extract_filename_from_headers(&headers), Some("test.txt".to_string()));
    }

    #[test]
    fn test_extract_filename_from_headers_quoted_filename() {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_DISPOSITION, HeaderValue::from_static("attachment; filename=\"quoted file.txt\""));
        assert_eq!(extract_filename_from_headers(&headers), Some("quoted file.txt".to_string()));
    }

    #[test]
    fn test_extract_filename_from_headers_encoded_filename() {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_DISPOSITION, HeaderValue::from_static("attachment; filename*=UTF-8''encoded%20file.txt"));
        assert_eq!(extract_filename_from_headers(&headers), Some("encoded file.txt".to_string()));
    }

    #[test]
    fn test_extract_filename_from_headers_encoded_filename_with_special_chars() {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_DISPOSITION, HeaderValue::from_static("attachment; filename*=UTF-8''f%C3%A9ile%20name%21.txt"));
        assert_eq!(extract_filename_from_headers(&headers), Some("féile name!.txt".to_string()));
    }

    #[test]
    fn test_extract_filename_from_headers_no_filename() {
        let headers = HeaderMap::new();
        assert_eq!(extract_filename_from_headers(&headers), None);
    }

    #[test]
    fn test_extract_filename_from_headers_empty_filename() {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_DISPOSITION, HeaderValue::from_static("attachment; filename="));
        assert_eq!(extract_filename_from_headers(&headers), None);
    }

    #[test]
    fn test_extract_filename_precedence_headers_over_url() {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_DISPOSITION, HeaderValue::from_static("attachment; filename=header_file.txt"));
        let url = "http://example.com/url_file.txt";
        assert_eq!(extract_filename(&headers, url), Some("header_file.txt".to_string()));
    }

    #[test]
    fn test_extract_filename_sanitization_invalid_chars() {
        let filename = "file/name:with?invalid\"chars*.txt";
        assert_eq!(sanitize_filename(filename), "file_name_with_invalid_chars_.txt".to_string());
    }

    #[test]
    fn test_extract_filename_sanitization_leading_dots() {
        let filename = "...file.txt";
        assert_eq!(sanitize_filename(filename), "file.txt".to_string());
    }

    #[test]
    fn test_extract_filename_sanitization_empty_after_sanitize() {
        let filename = "///";
        assert_eq!(sanitize_filename(filename), "unnamed_file".to_string());
    }
}