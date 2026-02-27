use crate::config::ServerConfig;
use crate::core::Response;
use crate::server::handler::{default_reason_phrase, guess_mime_type};
use std::fs;
use std::path::Path;

fn load_custom_error_page(status: u16, config: &ServerConfig) -> Option<Response> {
    let err_cfg = config.errors.get(&status.to_string())?;
    let filename = match &err_cfg.filename {
        Some(name) => name,
        None => {
            eprintln!(
                "Warning: custom error {} configured without a filename; using default response",
                status
            );
            return None;
        }
    };

    let path = Path::new(&config.root).join("errors").join(filename);
    match fs::read(&path) {
        Ok(bytes) => {
            let reason = default_reason_phrase(status);
            let filename = path.to_string_lossy();
            let mime = guess_mime_type(&filename);
            Some(
                Response::new(status, reason)
                    .header("Content-Type", mime)
                    .with_body(bytes),
            )
        }
        Err(e) => {
            eprintln!(
                "Warning: failed to read custom error page '{}': {}",
                path.display(),
                e
            );
            None
        }
    }
}

/// Return an error response using a custom page if configured under `root/errors`.
pub fn error_response_from_config(status: u16, config: &ServerConfig) -> Response {
    if let Some(response) = load_custom_error_page(status, config) {
        return response;
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

#[cfg(test)]
mod tests {
    use super::error_response_from_config;
    use crate::config::{RouteConfig, ServerConfig};
    use std::collections::HashMap;

    fn base_config() -> ServerConfig {
        ServerConfig {
            server_address: "127.0.0.1".to_string(),
            ports: vec![8080],
            server_name: Some("localhost".to_string()),
            root: "root".to_string(),
            routes: HashMap::new(),
            cgi_handlers: HashMap::new(),
            errors: HashMap::new(),
            admin_access: false,
        }
    }

    #[test]
    fn fallback_response_has_expected_status_and_content_type() {
        let config = base_config();
        let response = error_response_from_config(404, &config);

        assert_eq!(response.status_code, 404);
        assert_eq!(
            response.headers.get("Content-Type").map(String::as_str),
            Some("text/html; charset=utf-8")
        );
    }

    #[test]
    fn missing_custom_error_filename_falls_back_to_default() {
        let mut config = base_config();
        config.errors.insert(
            "500".to_string(),
            RouteConfig {
                filename: None,
                directory: None,
                directory_listing: false,
                methods: None,
                redirect: None,
                upload_dir: None,
            },
        );

        let response = error_response_from_config(500, &config);
        let body = String::from_utf8_lossy(&response.body);

        assert_eq!(response.status_code, 500);
        assert!(body.contains("500 Internal Server Error"));
    }
}
