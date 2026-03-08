use std::collections::HashMap;
use std::io::{self, ErrorKind};
use std::net::{SocketAddr, TcpListener};

use crate::config::ServerConfig;
use crate::ClientConnection;

#[derive(Debug)]
pub struct ServerSocket {
    pub addr: SocketAddr,
    pub listener: TcpListener,
    pub configs: Vec<ServerConfig>,
}

impl ServerSocket {
    /// Create a new non-blocking socket bound to the address and associate it with config.
    pub fn try_bind(addr: SocketAddr, configs: Vec<ServerConfig>) -> io::Result<Self> {
        let listener = TcpListener::bind(addr)?;
        listener.set_nonblocking(true)?;
        println!("[+] Bound to {}", addr);

        Ok(Self {
            addr,
            listener,
            configs,
        })
    }

    pub fn resolve_config(&self, server_name: Option<&str>) -> &ServerConfig {
        if let Some(name) = server_name {
            for config in &self.configs {
                if let Some(cfg_name) = &config.server_name {
                    if cfg_name == name {
                        return config;
                    }
                }
            }
        }

        // Fallback: first server config
        &self.configs[0]
    }

    /// Accepts all pending connections (non-blocking), returns new clients.
    pub fn try_accept(&self) -> Vec<ClientConnection> {
        let mut new_clients = Vec::new();

        loop {
            match self.listener.accept() {
                Ok((stream, peer_addr)) => {
                    /* println!(
                        "[*] Accepted connection from {} on {}",
                        peer_addr, self.addr
                    ); */
                    match ClientConnection::new(stream, peer_addr) {
                        Ok(client) => new_clients.push(client),
                        Err(e) => eprintln!("[!] Failed to create client connection: {}", e),
                    }
                }
                Err(ref e) if e.kind() == ErrorKind::WouldBlock => break,
                Err(e) => {
                    eprintln!("[!] Error accepting on {}: {}", self.addr, e);
                    break;
                }
            }
        }

        new_clients
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn minimal_config(server_name: Option<&str>) -> ServerConfig {
        ServerConfig {
            server_address: "127.0.0.1".to_string(),
            ports: vec![8080],
            server_name: server_name.map(String::from),
            root: "/tmp".to_string(),
            routes: HashMap::new(),
            cgi_handlers: HashMap::new(),
            errors: HashMap::new(),
            admin_access: false,
        }
    }

    #[test]
    fn try_bind_succeeds_with_port_zero() {
        let addr = SocketAddr::from(([127, 0, 0, 1], 0));
        let configs = vec![minimal_config(Some("default"))];
        let socket = ServerSocket::try_bind(addr, configs).unwrap();
        assert_eq!(socket.addr.ip(), std::net::IpAddr::from([127, 0, 0, 1]));
        assert_eq!(socket.configs.len(), 1);
    }

    #[test]
    fn resolve_config_returns_matching_config_by_name() {
        let addr = SocketAddr::from(([127, 0, 0, 1], 0));
        let configs = vec![minimal_config(Some("alpha")), minimal_config(Some("beta"))];
        let socket = ServerSocket::try_bind(addr, configs).unwrap();
        let cfg_a = socket.resolve_config(Some("alpha"));
        let cfg_b = socket.resolve_config(Some("beta"));
        assert_eq!(cfg_a.server_name.as_deref(), Some("alpha"));
        assert_eq!(cfg_b.server_name.as_deref(), Some("beta"));
    }

    #[test]
    fn resolve_config_fallback_to_first_when_unknown_name() {
        let addr = SocketAddr::from(([127, 0, 0, 1], 0));
        let configs = vec![minimal_config(Some("only")), minimal_config(Some("other"))];
        let socket = ServerSocket::try_bind(addr, configs).unwrap();
        let cfg = socket.resolve_config(Some("unknown"));
        assert_eq!(cfg.server_name.as_deref(), Some("only"));
    }

    #[test]
    fn resolve_config_fallback_to_first_when_none() {
        let addr = SocketAddr::from(([127, 0, 0, 1], 0));
        let configs = vec![
            minimal_config(Some("first")),
            minimal_config(Some("second")),
        ];
        let socket = ServerSocket::try_bind(addr, configs).unwrap();
        let cfg = socket.resolve_config(None);
        assert_eq!(cfg.server_name.as_deref(), Some("first"));
    }
}
