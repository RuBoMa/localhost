use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::{fs, path::Path};

#[derive(Debug, Deserialize)]
pub struct Config {
    pub servers: Vec<ServerConfig>,

    #[serde(default = "default_timeout_secs")]
    pub client_timeout_secs: u64,

    #[serde(default)]
    pub admin: AdminConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub server_address: String,
    pub ports: Vec<u16>,
    pub server_name: Option<String>,

    #[serde(default)]
    pub root: String,

    #[serde(default)]
    pub routes: HashMap<String, RouteConfig>,

    /// Map file extensions to CGI interpreter/command
    #[serde(default)]
    pub cgi_handlers: HashMap<String, String>,

    /// HTTP status code -> custom error file
    #[serde(default)]
    pub errors: HashMap<String, RouteConfig>,

    #[serde(default)]
    pub admin_access: bool,
}

fn default_timeout_secs() -> u64 {
    30 // default 30 seconds if not specified
}

impl Config {
    pub fn validate(&self) -> Result<(), String> {
        let mut seen_servers: HashSet<(u16, String)> = HashSet::new();

        for server in &self.servers {
            if server.ports.is_empty() {
                return Err(format!("Server at {} has no ports", server.server_address));
            }

            if server.root.trim().is_empty() {
                return Err(format!(
                    "Server at {} must have a non-empty 'root' directory defined",
                    server.server_address
                ));
            }

            if !Path::new(&server.root).is_dir() {
                return Err(format!("Root directory '{}' does not exist", server.root));
            }

            for &port in &server.ports {
                // Empty string for nameless server
                let name = server.server_name.clone().unwrap_or_default();

                let key = (port, name.clone());

                if !seen_servers.insert(key.clone()) {
                    if name.is_empty() {
                        return Err(format!(
                            "Duplicate nameless server configured on port {}",
                            port
                        ));
                    } else {
                        return Err(format!(
                            "Duplicate server name '{}' configured on port {}",
                            name, port
                        ));
                    }
                }
            }

            for (route, cfg) in &server.routes {
                if !route.starts_with("/") {
                    eprintln!("Warning: route '{}' should start with '/'", route);
                }

                // A valid route must define at least one of these
                if cfg.filename.is_none()
                    && cfg.directory.is_none()
                    && cfg.redirect.is_none()
                    && cfg.upload_dir.is_none()
                {
                    eprintln!(
                        "Warning: Route '{}' has no directory, redirect, upload_dir, or filename defined. Default index will be served.",
                        route
                    );
                }

                // Check file existence
                if let Some(filename) = &cfg.filename {
                    let full_path = Path::new(&server.root).join(filename);
                    if !full_path.exists() {
                        eprintln!(
                            "Warning: route '{}' points to missing file: {}",
                            route,
                            full_path.display()
                        );
                    }
                }

                // Check directory existence
                if let Some(directory) = &cfg.directory {
                    if route == "/" {
                        return Err("Route '/' cannot serve a directory — use a subpath like '/files' instead.".to_string());
                    }

                    let full_path = Path::new(&server.root).join(directory);
                    if !full_path.exists() || !full_path.is_dir() {
                        eprintln!(
                            "Warning: route '{}' points to missing or invalid directory: {}",
                            route,
                            full_path.display()
                        );
                    }
                }

                // Validate upload dir (we create it later if needed)
                if let Some(upload_dir) = &cfg.upload_dir {
                    let path = Path::new(upload_dir);
                    if path.exists() && !path.is_dir() {
                        return Err(format!(
                            "Route '{}' defines an upload_dir that exists but is not a directory: {}",
                            route,
                            path.display()
                        ));
                    }
                }
            }

            // Validate custom error files under root/errors
            if !server.errors.is_empty() {
                let errors_dir = std::path::Path::new(&server.root).join("errors");
                for (code, cfg) in &server.errors {
                    // best-effort code parse to notify users early
                    if code.parse::<u16>().is_err() {
                        eprintln!("Warning: error code '{}' is not a valid u16", code);
                    }

                    let Some(filename) = &cfg.filename else {
                        eprintln!("Warning: custom error {} has no filename configured", code);
                        continue;
                    };

                    let full_path = errors_dir.join(filename);
                    if !full_path.exists() {
                        eprintln!(
                            "Warning: custom error {} file not found: {}",
                            code,
                            full_path.display()
                        );
                    }
                }
            }
        }

        Ok(())
    }

    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let content =
            fs::read_to_string(path).map_err(|e| format!("Failed to read config: {}", e))?;

        let config: Config =
            toml::from_str(&content).map_err(|e| format!("Failed to parse TOML: {}", e))?;

        config.validate()?;

        Ok(config)
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct RouteConfig {
    #[serde(default)]
    pub filename: Option<String>, // for single file

    #[serde(default)]
    pub directory: Option<String>, // for directory mapping

    #[serde(default)]
    pub directory_listing: bool, // default to false

    #[serde(default)]
    pub methods: Option<Vec<String>>, // allowed methods

    #[serde(default)]
    pub redirect: Option<RedirectConfig>, // optional redirect

    #[serde(default)]
    pub upload_dir: Option<String>,
}

impl RouteConfig {
    pub fn check_method(&self, method: &str) -> Result<(), String> {
        if let Some(allowed) = &self.methods {
            if !allowed.iter().any(|m| m.eq_ignore_ascii_case(method)) {
                return Err(allowed.join(", "));
            }
        }
        Ok(())
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct RedirectConfig {
    pub to: String, // Target URL or path

    #[serde(default = "default_redirect_code")]
    pub code: u16, // e.g., 301 or 302
}

fn default_redirect_code() -> u16 {
    302 // Default to 302 Found
}

#[derive(Debug, Default, Deserialize, Clone)]
pub struct AdminConfig {
    #[serde(default = "default_admin_username")]
    pub username: String,

    #[serde(default = "default_admin_password")]
    pub password: String,
}

fn default_admin_username() -> String {
    "admin".to_string()
}

fn default_admin_password() -> String {
    "password123".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Create a temporary directory for tests that require a real root path.
    fn temp_root() -> std::path::PathBuf {
        let dir = std::env::temp_dir().join("localhost_config_test");
        let _ = std::fs::create_dir_all(&dir);
        dir
    }

    #[test]
    fn test_parse_minimal_config_defaults() {
        let root = temp_root();
        let toml = format!(
            r#"
            [[servers]]
            server_address = "127.0.0.1"
            ports = [8080]
            root = "{}"
            "#,
            root.display().to_string().replace('\\', "\\\\")
        );
        let config: Config = toml::from_str(&toml).unwrap();
        assert_eq!(config.client_timeout_secs, 30);
        assert_eq!(config.admin.username, "admin");
        assert_eq!(config.admin.password, "password123");
        assert_eq!(config.servers.len(), 1);
        assert_eq!(config.servers[0].ports, vec![8080]);
        assert!(config.servers[0].server_name.is_none());
    }

    #[test]
    fn test_parse_config_with_custom_timeout_and_admin() {
        let root = temp_root();
        let toml = format!(
            r#"
            client_timeout_secs = 60
            [admin]
            username = "custom_user"
            password = "custom_pass"

            [[servers]]
            server_address = "127.0.0.1"
            ports = [9000]
            root = "{}"
            "#,
            root.display().to_string().replace('\\', "\\\\")
        );
        let config: Config = toml::from_str(&toml).unwrap();
        assert_eq!(config.client_timeout_secs, 60);
        assert_eq!(config.admin.username, "custom_user");
        assert_eq!(config.admin.password, "custom_pass");
    }

    #[test]
    fn test_validate_empty_ports_fails() {
        let root = temp_root();
        let toml = format!(
            r#"
            [[servers]]
            server_address = "127.0.0.1"
            ports = []
            root = "{}"
            "#,
            root.display().to_string().replace('\\', "\\\\")
        );
        let config: Config = toml::from_str(&toml).unwrap();
        let err = config.validate().unwrap_err();
        assert!(err.contains("no ports"), "expected 'no ports' in: {}", err);
    }

    #[test]
    fn test_validate_empty_root_fails() {
        let toml = r#"
            [[servers]]
            server_address = "127.0.0.1"
            ports = [8080]
            root = "    "
        "#;
        let config: Config = toml::from_str(toml).unwrap();
        let err = config.validate().unwrap_err();
        assert!(
            err.contains("non-empty 'root'"),
            "expected 'non-empty root' in: {}",
            err
        );
    }

    #[test]
    fn test_validate_root_not_a_directory_fails() {
        let toml = r#"
            [[servers]]
            server_address = "127.0.0.1"
            ports = [8080]
            root = "/nonexistent_path_12345_xyz"
        "#;
        let config: Config = toml::from_str(toml).unwrap();
        let err = config.validate().unwrap_err();
        assert!(
            err.contains("does not exist"),
            "expected 'does not exist' in: {}",
            err
        );
    }

    #[test]
    fn test_validate_duplicate_nameless_port_fails() {
        let root = temp_root();
        let toml = format!(
            r#"
            [[servers]]
            server_address = "127.0.0.1"
            ports = [8080]
            root = "{}"

            [[servers]]
            server_address = "127.0.0.1"
            ports = [8080]
            root = "{}"
            "#,
            root.display().to_string().replace('\\', "\\\\"),
            root.display().to_string().replace('\\', "\\\\")
        );
        let config: Config = toml::from_str(&toml).unwrap();
        let err = config.validate().unwrap_err();
        assert!(
            err.contains("Duplicate") && err.contains("8080"),
            "expected duplicate port in: {}",
            err
        );
    }

    #[test]
    fn test_validate_route_slash_with_directory_fails() {
        let root = temp_root();
        let sub = root.join("files");
        let _ = std::fs::create_dir_all(&sub);
        let toml = format!(
            r#"
            [[servers]]
            server_address = "127.0.0.1"
            ports = [8080]
            root = "{}"
            [servers.routes."/"]
            directory = "files"
            "#,
            root.display().to_string().replace('\\', "\\\\")
        );
        let config: Config = toml::from_str(&toml).unwrap();
        let err = config.validate().unwrap_err();
        assert!(
            err.contains("Route '/' cannot serve a directory"),
            "expected route '/' directory error in: {}",
            err
        );
    }

    #[test]
    fn test_validate_upload_dir_not_directory_fails() {
        let root = temp_root();
        let file_path = root.join("upload_file");
        let _ = std::fs::File::create(&file_path);
        let toml = format!(
            r#"
            [[servers]]
            server_address = "127.0.0.1"
            ports = [8080]
            root = "{}"
            [servers.routes."/upload"]
            upload_dir = "{}"
            "#,
            root.display().to_string().replace('\\', "\\\\"),
            file_path.display().to_string().replace('\\', "\\\\")
        );
        let config: Config = toml::from_str(&toml).unwrap();
        let err = config.validate().unwrap_err();
        assert!(
            err.contains("upload_dir") && err.contains("not a directory"),
            "expected upload_dir error in: {}",
            err
        );
    }

    #[test]
    fn test_validate_valid_config_succeeds() {
        let root = temp_root();
        let toml = format!(
            r#"
            [[servers]]
            server_address = "127.0.0.1"
            ports = [8080]
            root = "{}"
            "#,
            root.display().to_string().replace('\\', "\\\\")
        );
        let config: Config = toml::from_str(&toml).unwrap();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_from_file_missing_fails() {
        let result = Config::from_file("/nonexistent_config_12345.toml");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("Failed to read config") || err.contains("read"));
    }

    #[test]
    fn test_from_file_invalid_toml_fails() {
        let root = temp_root();
        let path = root.join("bad.toml");
        std::fs::write(&path, "invalid toml [[[[").unwrap();
        let result = Config::from_file(&path);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("parse") || err.contains("TOML"));
    }

    #[test]
    fn test_from_file_valid_succeeds() {
        let root = temp_root();
        let path = root.join("valid.toml");
        let toml = format!(
            r#"
            [[servers]]
            server_address = "127.0.0.1"
            ports = [8080]
            root = "{}"
            "#,
            root.display().to_string().replace('\\', "\\\\")
        );
        std::fs::write(&path, &toml).unwrap();
        let config = Config::from_file(&path).unwrap();
        assert_eq!(config.servers.len(), 1);
        assert_eq!(config.servers[0].ports[0], 8080);
    }

    #[test]
    fn test_route_config_check_method_allowed() {
        let cfg = RouteConfig {
            filename: None,
            directory: None,
            directory_listing: false,
            methods: Some(vec!["GET".to_string(), "POST".to_string()]),
            redirect: None,
            upload_dir: None,
        };
        assert!(cfg.check_method("GET").is_ok());
        assert!(cfg.check_method("get").is_ok());
        assert!(cfg.check_method("POST").is_ok());
    }

    #[test]
    fn test_route_config_check_method_disallowed() {
        let cfg = RouteConfig {
            filename: None,
            directory: None,
            directory_listing: false,
            methods: Some(vec!["GET".to_string()]),
            redirect: None,
            upload_dir: None,
        };
        let err = cfg.check_method("POST").unwrap_err();
        assert_eq!(err, "GET");
    }

    #[test]
    fn test_route_config_check_method_none_always_ok() {
        let cfg = RouteConfig {
            filename: None,
            directory: None,
            directory_listing: false,
            methods: None,
            redirect: None,
            upload_dir: None,
        };
        assert!(cfg.check_method("GET").is_ok());
        assert!(cfg.check_method("DELETE").is_ok());
    }

    #[test]
    fn test_redirect_config_default_code() {
        let toml = r#"
            to = "/other"
        "#;
        let r: RedirectConfig = toml::from_str(toml).unwrap();
        assert_eq!(r.to, "/other");
        assert_eq!(r.code, 302);
    }

    #[test]
    fn test_redirect_config_custom_code() {
        let toml = r#"
            to = "/moved"
            code = 301
        "#;
        let r: RedirectConfig = toml::from_str(toml).unwrap();
        assert_eq!(r.to, "/moved");
        assert_eq!(r.code, 301);
    }
}
