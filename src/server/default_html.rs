use crate::core::Response;

pub fn default_welcome_response() -> Response {
    Response::new(200, "OK")
        .header("Content-Type", "text/html")
        .with_body(
            r#"<!DOCTYPE html>
<html>
<head><title>localhost</title></head>
<body>
  <h1>Welcome</h1>
  <p>Your server is running, but no routes or root directory were configured.</p>
</body>
</html>
"#,
        )
}

pub fn default_404_response() -> Response {
    Response::new(404, "Not Found")
        .header("Content-Type", "text/html")
        .with_body(
            r#"<!DOCTYPE html>
<html>
<head><title>404 Not Found</title></head>
<body>
  <h1>404 Not Found</h1>
  <p>The requested resource could not be found.</p>
</body>
</html>
"#,
        )
}

pub fn default_method_not_allowed_response(allowed: Option<&str>) -> Response {
    let mut response = Response::new(405, "Method Not Allowed")
        .header("Content-Type", "text/html")
        .with_body(r#"<!DOCTYPE html>
<html>
<head><title>405 Method Not Allowed</title></head>
<body>
  <h1>405 Method Not Allowed</h1>
  <p>The requested method is not allowed for the specified resource.</p>
</body>
</html>"#);

    if let Some(methods) = allowed {
        response = response.header("Allow", methods);
    }

    response
}
