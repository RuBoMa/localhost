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
    pub fn parse(raw: &str) -> Option<Self> {
        let mut lines = raw.lines();

        let request_line = lines.next()?;
        let mut parts = request_line.split_whitespace();
        let method = parts.next()?.to_string();
        let uri = parts.next()?.to_string();
        let version = parts.next()?.to_string();

        let mut headers = HashMap::new();

        for line in lines {
            if line.is_empty() {
                break;
            }
            if let Some((key, value)) = line.split_once(":") {
                headers.insert(key.trim().to_string(), value.trim().to_string());
            }
        }

        // For now, assume no body
        Some(Request {
            method,
            uri,
            version,
            headers,
            body: Vec::new(),
        })
    }
}
