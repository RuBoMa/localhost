use std::path::Path;
use std::fs;
use crate::core::Response;
use crate::server::default_404_response;
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
        Err(_) => {
            default_404_response()
        }
    }
}