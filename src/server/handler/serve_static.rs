use crate::core::Response;
use crate::server::default_html::{default_error_response};
use std::fs;
use std::path::Path;
use super::utils::guess_mime_type;

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