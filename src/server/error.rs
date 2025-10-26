use crate::core::Response;
use crate::config::ServerConfig;
use std::fs;
use std::path::Path;
use crate::server::handler::utils::{guess_mime_type, default_reason_phrase};

/// Return an error response using a custom page if configured under `root/errors`.
pub fn error_response_from_config(status: u16, config: &ServerConfig) -> Response {
    if let Some(err_cfg) = config.errors.get(&status.to_string()) {
        let path = Path::new(&config.root).join("errors").join(&err_cfg.filename);
        if let Ok(bytes) = fs::read(&path) {
            let reason = default_reason_phrase(status);
            let mime = guess_mime_type(path.to_string_lossy().as_ref());
            return Response::new(status, reason)
                .header("Content-Type", mime)
                .with_body(bytes);
        }
    }
    // Fallback if no default error page has been defined
    let reason = default_reason_phrase(status);
    let title = format!("{} {}", status, reason);
    let body = format!(
        "<!DOCTYPE html>\n<html>\n<head><meta charset=\"utf-8\"><title>{}</title></head>\n<body>\n  <h1>{}</h1>\n</body>\n</html>\n",
        title, title
    );
    Response::new(status, reason)
        .header("Content-Type", "text/html; charset=utf-8")
        .with_body(body)
}
