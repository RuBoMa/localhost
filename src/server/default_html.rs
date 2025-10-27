use std::collections::HashMap;
use crate::config::RouteConfig;
use crate::core::Response;

pub fn default_index_response(routes: &HashMap<String, RouteConfig>) -> Response {
    let mut body = String::from(r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>Registered Routes</title>
    <link rel="stylesheet" type="text/css" href="/static/style.css">
</head>
<body>
    <h1>Registered Routes</h1>
    <ul>
"#);

    for (route, cfg) in routes {
        let methods = if let Some(methods) = &cfg.methods {
            methods.join(", ")
        } else {
            "ALL".to_string()
        };

        body.push_str(&format!(
            r#"    <li>[{methods}] - <a href="{route}">{route}</a></li>"#,
            route = route,
            methods = methods
        ));
        body.push('\n');
    }

    body.push_str(r#"  </ul>
</body>
</html>
"#);

    Response::new(200, "OK")
        .header("Content-Type", "text/html")
        .with_body(body)
}

pub fn default_error_response(status_code: u16, reason: &str, message: Option<&str>) -> Response {
    let message = message.unwrap_or("An unexpected error occurred.");

    Response::new(status_code, reason)
        .header("Content-Type", "text/html")
        .with_body(format!(
            r#"<!DOCTYPE html>
<html>
<head><title>{code} {reason}</title></head>
<body>
  <h1>{code} {reason}</h1>
  <p>{msg}</p>
</body>
</html>
"#,
            code = status_code,
            reason = reason,
            msg = message
        ))
}

pub fn default_400_response() -> Response {
    default_error_response(400, "Bad Request", Some("The request could not be understood by the server due to malformed syntax."))
}

pub fn default_403_response() -> Response {
    default_error_response(403, "Forbidden", Some("You do not have permission to access the requested resource."))
}

pub fn default_404_response() -> Response {
    default_error_response(404, "Not Found", Some("The requested resource could not be found."))
}

pub fn default_405_response(allowed_methods: Option<&str>) -> Response {
    let mut response = default_error_response(405, "Method Not Allowed", Some("The method specified in the request is not allowed for the resource."));

    if let Some(allowed) = allowed_methods {
        response = response.header("Allow", allowed);
    }

    response
}

pub fn _default_413_response() -> Response {
    default_error_response(413, "Payload Too Large", Some("The request is larger than the server is willing or able to process."))
}

pub fn default_500_response(reason: Option<&str>) -> Response {
    default_error_response(500, "Internal Server Error", reason)
}