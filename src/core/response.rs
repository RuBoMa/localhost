use std::collections::HashMap;
use std::fmt::Write as FmtWrite;

#[derive(Debug)]
pub struct Response {
    pub status_code: u16,
    pub reason_phrase: String,
    pub headers: HashMap<String, String>,
    pub cookies: Vec<String>,
    pub body: Vec<u8>,
}

impl Response {
    pub fn new(status_code: u16, reason_phrase: &str) -> Self {
        Self {
            status_code,
            reason_phrase: reason_phrase.to_string(),
            headers: HashMap::new(),
            cookies: Vec::new(),
            body: Vec::new(),
        }
    }

    pub fn with_body(mut self, body: impl Into<Vec<u8>>) -> Self {
        let body_bytes = body.into();
        self.headers
            .insert("Content-Length".to_string(), body_bytes.len().to_string());
        self.body = body_bytes;
        self
    }

    pub fn header(mut self, key: &str, value: &str) -> Self {
        self.headers.insert(key.to_string(), value.to_string());
        self
    }

    /// Set a cookie with optional attributes
    pub fn set_cookie(
        mut self,
        name: &str,
        value: &str,
        path: Option<&str>,
        max_age: Option<u64>,
        http_only: bool,
    ) -> Self {
        let mut cookie = format!("{}={}", name, value);
        if let Some(p) = path {
            cookie.push_str(&format!("; Path={}", p));
        }
        if let Some(age) = max_age {
            cookie.push_str(&format!("; Max-Age={}", age));
        }
        if http_only {
            cookie.push_str("; HttpOnly");
        }
        self.cookies.push(cookie);
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

        // Cookies
        for cookie in &self.cookies {
            let _ = write!(response, "Set-Cookie: {}\r\n", cookie);
        }

        // End of headers
        response.push_str("\r\n");

        let mut bytes = response.into_bytes();
        bytes.extend_from_slice(&self.body);
        bytes
    }

    pub fn redirect(location: String, status_code: u16) -> Self {
        let reason_phrase = match status_code {
            301 => "Moved Permanently",
            302 => "Found",
            303 => "See Other",
            307 => "Temporary Redirect",
            308 => "Permanent Redirect",
            _ => "Redirect",
        }
        .to_string();

        let mut headers = HashMap::new();
        headers.insert("Location".to_string(), location.clone());
        headers.insert("Content-Type".to_string(), "text/html".to_string());

        let body = format!(
            "<html><body><h1>{} Redirect</h1><p>Redirecting to <a href=\"{}\">{}</a></p></body></html>",
            status_code, location, location
        );

        Self {
            status_code,
            reason_phrase,
            headers,
            cookies: Vec::new(),
            body: body.into_bytes(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_sets_status_and_reason() {
        let r = Response::new(200, "OK");
        assert_eq!(r.status_code, 200);
        assert_eq!(r.reason_phrase, "OK");
        assert!(r.headers.is_empty());
        assert!(r.body.is_empty());
    }

    #[test]
    fn with_body_sets_content_length_and_body() {
        let r = Response::new(200, "OK").with_body(b"hello");
        assert_eq!(r.body, b"hello");
        assert_eq!(r.headers.get("Content-Length"), Some(&"5".to_string()));
    }

    #[test]
    fn header_adds_header() {
        let r = Response::new(200, "OK").header("X-Custom", "value");
        assert_eq!(r.headers.get("X-Custom"), Some(&"value".to_string()));
    }

    #[test]
    fn set_cookie_basic() {
        let r = Response::new(200, "OK").set_cookie("sid", "abc", None, None, false);
        assert_eq!(r.cookies.len(), 1);
        assert_eq!(r.cookies[0], "sid=abc");
    }

    #[test]
    fn set_cookie_with_path_and_http_only() {
        let r = Response::new(200, "OK").set_cookie("sid", "xyz", Some("/"), None, true);
        assert!(r.cookies[0].contains("sid=xyz"));
        assert!(r.cookies[0].contains("Path=/"));
        assert!(r.cookies[0].contains("HttpOnly"));
    }

    #[test]
    fn set_cookie_with_max_age() {
        let r = Response::new(200, "OK").set_cookie("s", "v", None, Some(3600), false);
        assert!(r.cookies[0].contains("Max-Age=3600"));
    }

    #[test]
    fn to_bytes_includes_status_headers_and_body() {
        let r = Response::new(200, "OK")
            .header("Content-Type", "text/plain")
            .with_body(b"body");
        let bytes = r.to_bytes();
        let s = std::str::from_utf8(&bytes).unwrap();
        assert!(s.starts_with("HTTP/1.1 200 OK\r\n"));
        assert!(s.contains("Content-Type: text/plain\r\n"));
        assert!(s.contains("Content-Length: 4\r\n"));
        assert!(s.ends_with("\r\n\r\nbody"));
    }

    #[test]
    fn to_bytes_includes_set_cookie() {
        let r = Response::new(200, "OK").set_cookie("k", "v", None, None, false);
        let bytes = r.to_bytes();
        let s = std::str::from_utf8(&bytes).unwrap();
        assert!(s.contains("Set-Cookie: k=v\r\n"));
    }

    #[test]
    fn redirect_302_has_location_and_found() {
        let r = Response::redirect("/other".to_string(), 302);
        assert_eq!(r.status_code, 302);
        assert_eq!(r.reason_phrase, "Found");
        assert_eq!(r.headers.get("Location"), Some(&"/other".to_string()));
        assert!(r.body.contains(&b"302"[..]));
        assert!(r.body.contains(&b"/other"[..]));
    }

    #[test]
    fn redirect_301_moved_permanently() {
        let r = Response::redirect("https://example.com".to_string(), 301);
        assert_eq!(r.status_code, 301);
        assert_eq!(r.reason_phrase, "Moved Permanently");
    }
}
