use crate::core::Response;
use crate::server::default_html::{default_error_response};
use std::fs;
use std::path::Path;

pub fn serve_static_file(path: &Path) -> Response {
    match fs::read(path) {
        Ok(contents) => {
            let filename = path.to_string_lossy();
            let mime = guess_mime_type(&filename);
            Response::new(200, "OK")
                .header("Content-Type", mime)
                .with_body(contents)
        }
        Err(_) => default_error_response(404, "Not found", None),
    }
}

/// Very basic MIME type guessing based on file extension.
/// Extend as needed for your use case.
pub fn guess_mime_type(filename: &str) -> &str {
    if let Some(ext) = filename.rsplit('.').next() {
        match ext {
            "html" => "text/html",
            "htm"  => "text/html",
            "css"  => "text/css",
            "js"   => "application/javascript",
            "json" => "application/json",
            "png"  => "image/png",
            "jpg" | "jpeg" => "image/jpeg",
            "gif"  => "image/gif",
            "svg"  => "image/svg+xml",
            "txt"  => "text/plain",
            "ico"  => "image/x-icon",
            "wasm" => "application/wasm",
            _ => "application/octet-stream", // unknown extension
        }
    } else {
        "application/octet-stream" // no extension
    }
}
