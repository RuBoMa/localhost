use crate::core::Response;
use crate::server::error_response_from_config;
use crate::config::ServerConfig;

use std::fs;
use std::path::Path;
use super::utils::guess_mime_type;

pub fn serve_static_file(path: &Path, config: &ServerConfig) -> Response {
    match fs::read(path) {
        Ok(contents) => {
            let filename = path.to_string_lossy();
            let mime = guess_mime_type(&filename);
            Response::new(200, "OK")
                .header("Content-Type", mime)
                .with_body(contents)
        }
        Err(_) => error_response_from_config(404, config),
    }
}