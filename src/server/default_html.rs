pub const DEFAULT_WELCOME_PAGE: &str = r#"
<!DOCTYPE html>
<html>
<head><title>localhost</title></head>
<body>
  <h1>Welcome</h1>
  <p>Your server is running, but no routes or root directory were configured.</p>
</body>
</html>
"#;

pub const DEFAULT_404_PAGE: &str = r#"
<!DOCTYPE html>
<html>
<head><title>404 Not Found</title></head>
<body>
  <h1>404 Not Found</h1>
  <p>The requested resource could not be found.</p>
</body>
</html>
"#;
