use std::collections::HashMap;

use crate::core::{extract_boundary, parse_multipart, MultipartPart};

#[derive(Debug)]
pub struct Request {
    pub method: String,
    pub uri: String,
    pub _version: String,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}

impl Request {
    pub fn parse(raw: &[u8]) -> Option<(Self, usize)> {
        // 1. Find header/body separator
        let header_end = raw.windows(4).position(|w| w == b"\r\n\r\n")?;

        // 2. Convert header part to UTF-8
        let header_str = std::str::from_utf8(&raw[..header_end]).ok()?;

        // 3. Body as raw bytes
        let body_part = &raw[header_end + 4..];

        // 4. Parse request line and headers
        let mut lines = header_str.lines();
        let request_line = lines.next()?;
        let mut parts = request_line.split_whitespace();
        let method = parts.next()?.to_string();
        let uri = parts.next()?.to_string();
        let version = parts.next()?.to_string();

        let mut headers = HashMap::new();
        for line in lines {
            if let Some((key, value)) = line.split_once(":") {
                headers.insert(key.trim().to_string(), value.trim().to_string());
            }
        }

        // 5. Parse body according to Content-Length / chunked
        let (body, consumed_body_len) = match parse_body(&headers, body_part) {
            Some((b, len)) => (b, len),
            None => return None, // Body incomplete; wait for more bytes
        };
        let consumed = header_end + 4 + consumed_body_len;

        Some((
            Request {
                method,
                uri,
                _version: version,
                headers,
                body,
            },
            consumed,
        ))
    }
    
    pub fn is_multipart(&self) -> bool {
        self.headers
            .get("Content-Type")
            .map_or(false, |ct| ct.starts_with("multipart/form-data"))
    }

    pub fn multipart_parts(&self) -> Option<Vec<MultipartPart>> {
        let boundary = extract_boundary(self)?;
        Some(parse_multipart(&self.body, &boundary))
    }
}

fn parse_body(headers: &HashMap<String, String>, raw_body: &[u8]) -> Option<(Vec<u8>, usize)> {
    // Handle chunked encoding
    if let Some(encoding) = headers.get("Transfer-Encoding") {
        if encoding.eq_ignore_ascii_case("chunked") {
            return parse_chunked_body(raw_body);
        }
    }

    // Use Content-Length if available
    if let Some(len_str) = headers.get("Content-Length") {
        if let Ok(len) = len_str.parse::<usize>() {
            if raw_body.len() < len {
                // Body incomplete, wait for more bytes
                return None;
            }
            return Some((raw_body[..len].to_vec(), len));
        }
    }

    // No Content-Length or chunked encoding: assume empty body
    Some((Vec::new(), 0))
}

fn parse_chunked_body(data: &[u8]) -> Option<(Vec<u8>, usize)> {
    let mut i = 0;
    let mut result = Vec::new();

    while i < data.len() {
        let size_end = data[i..].windows(2).position(|w| w == b"\r\n")?;
        let size_line = std::str::from_utf8(&data[i..i + size_end]).ok()?;
        let size = usize::from_str_radix(size_line.trim(), 16).ok()?;

        i += size_end + 2;

        if size == 0 {
            if i + 2 > data.len() {
                return None; // malformed trailing CRLF
            }
            if &data[i..i + 2] != b"\r\n" {
                return None; // malformed trailing CRLF
            }

            i += 2; // consume trailing CRLF
            break;
        }

        // Check if chunk data + trailing CRLF available
        if i + size + 2 > data.len() {
            return None;
        }

        // Copy chunk data
        result.extend_from_slice(&data[i..i + size]);
        i += size;

        // Verify chunk ends with CRLF
        if &data[i..i + 2] != b"\r\n" {
            return None;
        }
        i += 2;
    }

    Some((result, i))
}
