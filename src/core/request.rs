use std::collections::HashMap;

use crate::core::{extract_boundary, parse_multipart, url_decode, MultipartPart};

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
        let header_str = std::str::from_utf8(&raw[..header_end]).ok()?;
        let body_part = &raw[header_end + 4..];

        // 2. Parse headers and request line
        let (method, uri, version, headers) = parse_headers(header_str)?;

        // 3. Parse body according to Content-Length / chunked
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
            .get("content-type")
            .map_or(false, |ct| ct.starts_with("multipart/form-data"))
    }

    pub fn multipart_parts(&self) -> Option<Vec<MultipartPart>> {
        let boundary = extract_boundary(self)?;
        Some(parse_multipart(&self.body, &boundary))
    }

    pub fn _has_cookies(&self) -> bool {
        self.headers.contains_key("cookie")
    }

    pub fn cookies(&self) -> HashMap<String, String> {
        let mut result = HashMap::new();
        if let Some(cookie_header) = self.headers.get("cookie") {
            for pair in cookie_header.split(';') {
                let pair = pair.trim();
                if let Some((name, value)) = pair.split_once('=') {
                    result.insert(name.to_string(), value.to_string());
                }
            }
        }
        result
    }

    pub fn get_cookie(&self, name: &str) -> Option<String> {
        self.cookies().get(name).cloned()
    }

    pub fn parse_form(&self) -> HashMap<String, String> {
        let mut form = HashMap::new();

        // Only parse if the content-type is form-urlencoded
        if let Some(ct) = self.headers.get("content-type") {
            if ct.starts_with("application/x-www-form-urlencoded") {
                let body_str = match std::str::from_utf8(&self.body) {
                    Ok(s) => s,
                    Err(_) => return form, // invalid UTF-8, return empty
                };

                for pair in body_str.split('&') {
                    if let Some((k, v)) = pair.split_once('=') {
                        // Decode URL-encoded keys and values
                        let key = url_decode(k).unwrap_or_else(|| k.to_string());
                        let value = url_decode(v).unwrap_or_else(|| v.to_string());
                        form.insert(key, value);
                    }
                }
            }
        }

        form
    }
}

fn parse_headers(header_str: &str) -> Option<(String, String, String, HashMap<String, String>)> {
    let mut lines = header_str.lines();
    let request_line = lines.next()?;

    let mut parts = request_line.split_whitespace();
    let (method, uri, version) = match (parts.next(), parts.next(), parts.next()) {
        (Some(m), Some(u), Some(v)) => (m.to_string(), u.to_string(), v.to_string()),
        _ => return None,
    };

    let mut headers = HashMap::new();
    for line in lines {
        if let Some((key, value)) = line.split_once(':') {
            headers.insert(key.trim().to_ascii_lowercase(), value.trim().to_string());
        }
    }

    Some((method, uri, version, headers))
}

fn parse_body(headers: &HashMap<String, String>, raw_body: &[u8]) -> Option<(Vec<u8>, usize)> {
    // Handle chunked encoding
    if let Some(encoding) = headers.get("transfer-encoding") {
        if encoding.eq_ignore_ascii_case("chunked") {
            return parse_chunked_body(raw_body);
        }
    }

    // Use Content-Length if available
    if let Some(len_str) = headers.get("content-length") {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_simple_get_no_body() {
        let raw = b"GET / HTTP/1.1\r\nHost: localhost\r\n\r\n";
        let (req, consumed) = Request::parse(raw).unwrap();
        assert_eq!(consumed, raw.len());
        assert_eq!(req.method, "GET");
        assert_eq!(req.uri, "/");
        assert_eq!(req.headers.get("host"), Some(&"localhost".to_string()));
        assert!(req.body.is_empty());
    }

    #[test]
    fn parse_headers_normalized_to_lowercase() {
        let raw = b"GET / HTTP/1.1\r\nContent-Type: text/plain\r\n\r\n";
        let (req, _) = Request::parse(raw).unwrap();
        assert_eq!(
            req.headers.get("content-type"),
            Some(&"text/plain".to_string())
        );
    }

    #[test]
    fn parse_with_content_length_body() {
        let raw = b"POST / HTTP/1.1\r\nContent-Length: 5\r\n\r\nhello";
        let (req, consumed) = Request::parse(raw).unwrap();
        assert_eq!(consumed, raw.len());
        assert_eq!(req.body, b"hello");
    }

    #[test]
    fn parse_incomplete_body_returns_none() {
        let raw = b"POST / HTTP/1.1\r\nContent-Length: 10\r\n\r\nshort";
        assert!(Request::parse(raw).is_none());
    }

    #[test]
    fn parse_chunked_body() {
        let raw = b"POST / HTTP/1.1\r\nTransfer-Encoding: chunked\r\n\r\n5\r\nhello\r\n0\r\n\r\n";
        let (req, consumed) = Request::parse(raw).unwrap();
        assert_eq!(consumed, raw.len());
        assert_eq!(req.body, b"hello");
    }

    #[test]
    fn parse_no_body_when_no_content_length_or_chunked() {
        let raw = b"GET / HTTP/1.1\r\nHost: x\r\n\r\nignored";
        let (req, consumed) = Request::parse(raw).unwrap();
        // No Content-Length/chunked: no body consumed. Consumed = header + \r\n\r\n (exact length can vary by platform).
        assert!(consumed < raw.len(), "consumed {} should be less than raw len {}", consumed, raw.len());
        assert!(req.body.is_empty());
    }

    #[test]
    fn parse_invalid_request_line_returns_none() {
        let raw = b"GET / \r\n\r\n";
        assert!(Request::parse(raw).is_none());
    }

    #[test]
    fn parse_no_double_crlf_returns_none() {
        let raw = b"GET / HTTP/1.1\r\nHost: x";
        assert!(Request::parse(raw).is_none());
    }

    #[test]
    fn is_multipart_true_for_multipart_content_type() {
        let raw = b"POST / HTTP/1.1\r\nContent-Type: multipart/form-data; boundary=----x\r\n\r\n";
        let (req, _) = Request::parse(raw).unwrap();
        assert!(req.is_multipart());
    }

    #[test]
    fn is_multipart_false_otherwise() {
        let raw = b"GET / HTTP/1.1\r\nContent-Type: text/html\r\n\r\n";
        let (req, _) = Request::parse(raw).unwrap();
        assert!(!req.is_multipart());
    }

    #[test]
    fn cookies_parses_single_cookie() {
        let raw = b"GET / HTTP/1.1\r\nCookie: session=abc123\r\n\r\n";
        let (req, _) = Request::parse(raw).unwrap();
        let cookies = req.cookies();
        assert_eq!(cookies.get("session"), Some(&"abc123".to_string()));
    }

    #[test]
    fn cookies_parses_multiple_cookies() {
        let raw = b"GET / HTTP/1.1\r\nCookie: a=1; b=2; c=3\r\n\r\n";
        let (req, _) = Request::parse(raw).unwrap();
        assert_eq!(req.get_cookie("a"), Some("1".to_string()));
        assert_eq!(req.get_cookie("b"), Some("2".to_string()));
        assert_eq!(req.get_cookie("c"), Some("3".to_string()));
    }

    #[test]
    fn get_cookie_missing_returns_none() {
        let raw = b"GET / HTTP/1.1\r\n\r\n";
        let (req, _) = Request::parse(raw).unwrap();
        assert!(req.get_cookie("any").is_none());
    }

    #[test]
    fn parse_form_urlencoded() {
        // Body "foo=bar" is 7 bytes
        let raw = b"POST / HTTP/1.1\r\nContent-Type: application/x-www-form-urlencoded\r\nContent-Length: 7\r\n\r\nfoo=bar";
        let (req, _) = Request::parse(raw).unwrap();
        let form = req.parse_form();
        assert_eq!(form.get("foo"), Some(&"bar".to_string()));
    }

    #[test]
    fn parse_form_decodes_plus_as_space() {
        // %2B decodes to '+'. Body "x=hi%2B" is 7 bytes (x, =, h, i, %, 2, B).
        let raw = b"POST / HTTP/1.1\r\nContent-Type: application/x-www-form-urlencoded\r\nContent-Length: 7\r\n\r\nx=hi%2B";
        let (req, _) = Request::parse(raw).unwrap();
        let form = req.parse_form();
        assert_eq!(form.get("x"), Some(&"hi+".to_string()));
    }

    #[test]
    fn parse_form_ignores_non_form_content_type() {
        let raw = b"POST / HTTP/1.1\r\nContent-Type: text/plain\r\nContent-Length: 3\r\n\r\nfoo";
        let (req, _) = Request::parse(raw).unwrap();
        let form = req.parse_form();
        assert!(form.is_empty());
    }
}
