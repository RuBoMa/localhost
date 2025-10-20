use std::collections::HashMap;
use std::fmt::Write as FmtWrite;

#[derive(Debug)]
pub struct Response {
    pub status_code: u16,
    pub reason_phrase: String,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}

impl Response {
    pub fn new(status_code: u16, reason_phrase: &str) -> Self {
        Self {
            status_code,
            reason_phrase: reason_phrase.to_string(),
            headers: HashMap::new(),
            body: Vec::new(),
        }
    }

    pub fn with_body(mut self, body: impl Into<Vec<u8>>) -> Self {
        let body_bytes = body.into();
        self.headers.insert("Content-Length".to_string(), body_bytes.len().to_string());
        self.body = body_bytes;
        self
    }

    pub fn header(mut self, key: &str, value: &str) -> Self {
        self.headers.insert(key.to_string(), value.to_string());
        self
    }

    /// Serialize the full HTTP response into bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut response = String::new();

        // Status line
        let _ = write!(
            response,
            "HTTP/1.1 {} {}\r\n",
            self.status_code, self.reason_phrase
        );

        // Headers
        for (key, value) in &self.headers {
            let _ = write!(response, "{}: {}\r\n", key, value);
        }

        // End of headers
        response.push_str("\r\n");

        let mut bytes = response.into_bytes();
        bytes.extend_from_slice(&self.body);
        bytes
    }
}
