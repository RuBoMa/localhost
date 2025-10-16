use serde::Deserialize;
use std::{fs, path::Path};

#[derive(Debug, Deserialize)]
pub struct Config {
    pub servers: Vec<ServerConfig>,
}

#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    pub server_address: String,
    pub ports: Vec<u16>,
    pub server_name: Option<String>,

    #[serde(default = "default_timeout_secs")]
    pub client_timeout_secs: u64,
}

fn default_timeout_secs() -> u64 {
    30 // default 30 seconds if not specified
}

impl Config {
    pub fn validate(&self) -> Result<(), String> {
        for server in &self.servers {
            if server.ports.is_empty() {
                return Err(format!(
                    "Server at {} has no ports defined",
                    server.server_address
                ));
            }
        }
        Ok(())
    }

    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config: {}", e))?;

        let config: Config = toml::from_str(&content)
            .map_err(|e| format!("Failed to parse TOML: {}", e))?;

        config.validate()?; // Run validation here

        Ok(config)
    }
}
