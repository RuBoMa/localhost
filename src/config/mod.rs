use serde::Deserialize;
use std::{fs, path::Path};
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub servers: Vec<ServerConfig>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub server_address: String,
    pub ports: Vec<u16>,
    pub server_name: Option<String>,
    
    #[serde(default)]
    pub root: String,

    #[serde(default = "default_timeout_secs")]
    pub client_timeout_secs: u64,
    
    #[serde(default)]
    pub routes: HashMap<String, FileRouteConfig>,
}

fn default_timeout_secs() -> u64 {
    30 // default 30 seconds if not specified
}

impl Config {
    pub fn validate(&self) -> Result<(), String> {
        for server in &self.servers {
            if server.ports.is_empty() {
                return Err(format!("Server at {} has no ports", server.server_address));
            }

            if server.routes.is_empty() {
                continue;
            }

            if !Path::new(&server.root).is_dir() {
                return Err(format!("Root directory '{}' does not exist", server.root));
            }

            for (route, cfg) in &server.routes {            
                if !route.starts_with("/") {
                    eprintln!("Warning: route '{}' should start with '/'", route);
                }

                if let Some(filename) = &cfg.filename {
                    let full_path = std::path::Path::new(&server.root).join(filename);
                    if !full_path.exists() {
                        eprintln!(
                            "Warning: route '{}' points to missing file: {}",
                            route,
                            full_path.display()
                        );
                    }
                } else if cfg.redirect.is_none() {
                    return Err(format!(
                        "Route '{}' must define either a filename or a redirect",
                        route
                    ));
                }
            }
        }

        Ok(())
    }

    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config: {}", e))?;

        let config: Config = toml::from_str(&content)
            .map_err(|e| format!("Failed to parse TOML: {}", e))?;

        config.validate()?;

        Ok(config)
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct FileRouteConfig {
    pub filename: Option<String>,
    pub methods: Option<Vec<String>>,

    #[serde(default)]
    pub redirect: Option<RedirectConfig>,
}


#[derive(Debug, Deserialize, Clone)]
pub struct RedirectConfig {
    pub to: String,               // Target URL or path
    #[serde(default = "default_redirect_code")]
    pub code: u16,                // e.g., 301 or 302
}

fn default_redirect_code() -> u16 {
    302 // Default to 302 Found
}