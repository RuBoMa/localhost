use std::collections::HashMap;

#[derive(Debug)]
pub struct Request {
    pub method: String,
    pub uri: String,
    pub version: String,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}

impl Request {
    pub fn parse(raw: &[u8]) -> Option<Self> {
        // Look for the split between headers and body: \r\n\r\n
        let request = std::str::from_utf8(raw).ok()?;
        let header_end = request.find("\r\n\r\n")?;

        let (header_part, body_part) = raw.split_at(header_end + 4);
        let header_str = std::str::from_utf8(&header_part[..header_end]).ok()?;

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

        let body = parse_body(&headers, body_part);

        Some(Request {
            method,
            uri,
            version,
            headers,
            body,
        })
    }

}

fn parse_body(headers: &HashMap<String, String>, raw_body: &[u8]) -> Vec<u8> {
    if let Some(encoding) = headers.get("Transfer-Encoding") {
        if encoding.eq_ignore_ascii_case("chunked") {
            return parse_chunked_body(raw_body).unwrap_or_default();
        }
    }

    // Fallback: Use Content-Length if available
    if let Some(len_str) = headers.get("Content-Length") {
        if let Ok(len) = len_str.parse::<usize>() {
            return raw_body[..len.min(raw_body.len())].to_vec();
        }
    }

    // No clear indicator, return entire body
    raw_body.to_vec()
}

fn parse_chunked_body(data: &[u8]) -> Option<Vec<u8>> {
    let mut i = 0;
    let mut result = Vec::new();

    while i < data.len() {
        let size_end = data[i..]
            .windows(2)
            .position(|w| w == b"\r\n")?;
        let size_line = std::str::from_utf8(&data[i..i + size_end]).ok()?;
        let size = usize::from_str_radix(size_line.trim(), 16).ok()?;

        i += size_end + 2;
        if size == 0 {
            break;
        }
        if i + size > data.len() {
            return None;
        }
        result.extend_from_slice(&data[i..i + size]);
        i += size + 2;
    }

    Some(result)
}
