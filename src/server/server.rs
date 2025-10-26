use std::net::{TcpListener, SocketAddr};
use std::io::{self, ErrorKind};
use std::{thread, time::Duration};
use std::collections::HashMap;
use std::path::Path;

use crate::Config;
use crate::config::ServerConfig;
use crate::ClientConnection;
use crate::core::{Response, Request};
use crate::server::error_response_from_config;
use crate::server::handler::{execute_handler};

#[derive(Debug)]
pub struct ServerSocket {
    pub addr: SocketAddr,
    pub listener: TcpListener,
    pub configs: Vec<ServerConfig>,
}

impl ServerSocket {
    /// Create a new non-blocking socket bound to the address and associate it with config.
    pub fn try_bind(
        addr: SocketAddr,
        configs: Vec<ServerConfig>,
    ) -> io::Result<Self> {
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

    pub fn timeout(&self, server_name: Option<&str>) -> Duration {
        Duration::from_secs(self.resolve_config(server_name)
            .client_timeout_secs)
    }

    /// Accepts all pending connections (non-blocking), returns new clients.
    pub fn try_accept(&self) -> Vec<ClientConnection> {
        let mut new_clients = Vec::new();

        loop {
            match self.listener.accept() {
                Ok((stream, peer_addr)) => {
                    println!("[*] Accepted connection from {} on {}", peer_addr, self.addr);
                    match ClientConnection::new(
                        stream,
                        peer_addr
                    ) {
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

#[derive(Debug)]
pub struct Server {
    sockets: Vec<ServerSocket>,
    clients: Vec<ClientConnection>,
}

impl Server {
    pub fn from_config(config: &Config) -> std::io::Result<Self> {
        let mut grouped: HashMap<SocketAddr, Vec<ServerConfig>> = HashMap::new();

        // Group configs by SocketAddr
        for server in &config.servers {
            for &port in &server.ports {
                let addr_str = format!("{}:{}", server.server_address, port);
                match addr_str.parse::<SocketAddr>() {
                    Ok(addr) => {
                        grouped.entry(addr)
                            .or_default()
                            .push(server.clone());
                    }
                    Err(e) => {
                        eprintln!("[!] Invalid address '{}': {}", addr_str, e);
                    }
                }
            }
        }

        let mut sockets = Vec::new();

        for (addr, configs) in grouped {
            match ServerSocket::try_bind(addr, configs) {
                Ok(socket) => sockets.push(socket),
                Err(e) => eprintln!("[!] Failed to bind {}: {}", addr, e),
            }
        }

        if sockets.is_empty() {
            return Err(std::io::Error::new(std::io::ErrorKind::Other, "No sockets bound"));
        }

        Ok(Server {
            sockets,
            clients: Vec::new(),
        })
    }

    fn handle_request(&self, request: &Request, client: &ClientConnection) -> Response {
        
        // Step 1: Get the socket the client connected to (by local_addr)
        let socket = match self.sockets.iter().find(|s| s.addr == client.local_addr) {
            Some(sock) => sock,
            None => {
                eprintln!("[!] No socket found for local addr {}", client.local_addr);
                return Response::new(500, "Internal Server Error")
                    .header("Content-Type", "text/html")
                    .with_body("<h1>500 Internal Server Error</h1><p>Socket not found.</p>");
            }
        };

        // Step 2: Extract server_name from the Host header
        let host_header = request.headers.get("Host").map(|s| s.as_str());
        let config = socket.resolve_config(host_header);

        let root_dir = Path::new(&config.root);

        // Step 3: Show default welcome page only if root directory doesn't exist
        if !root_dir.exists() {
            return Response::new(200, "OK")
                .header("Content-Type", "text/html")
                .with_body(r#"
<!DOCTYPE html>
<html>
<head><title>localhost</title></head>
<body>
  <h1>Welcome</h1>
  <p>Your server is running, but no routes or root directory were configured.</p>
</body>
</html>
"#);
        }

        // Step 4: Route matching
        if let Some(route_cfg) = config.routes.get(&request.uri) {
            let full_path = root_dir.join(&route_cfg.filename);
            execute_handler(&full_path, request, config, client.local_addr.port())
        } else {
            // Route not defined in config, but root exists
            error_response_from_config(404, config)
        }
    }

    fn handle_client(&mut self, client: &mut ClientConnection) -> io::Result<bool> {
        match client.read_into_buffer() {
            Ok(n) => {
                if n == 0 {
                    println!("[*] Client {} closed the connection", client.peer_addr);
                    return Ok(false);
                }

                client.refresh_activity();

                if let Some(request) = client.parse_request() {
                    println!("--- Parsed Request from {} ---\n{:#?}", client.peer_addr, request);

                    let response = self.handle_request(&request, &client);
                    client.send_response(response)?; // Clean + readable

                    // TODO: inspect request headers to decide keep-alive or close
                }

                Ok(true)
            }
            Err(e) => {
                eprintln!("[!] Error reading from {}: {}", client.peer_addr, e);
                Ok(false)
            }
        }
    }

    pub fn poll(&mut self) {
        for socket in &self.sockets {
            let new_clients = socket.try_accept();
            self.clients.extend(new_clients);
        }

        let mut i = 0;
        while i < self.clients.len() {
            let mut client = self.clients.swap_remove(i);

            let keep = match self.handle_client(&mut client) {
                Ok(keep) => keep,
                Err(e) => {
                    eprintln!("[!] Client error: {}", e);
                    false
                }
            };

            if keep {
                self.clients.push(client);
            }

            if keep {
                i += 1;
            }
        }
    }
    
    pub fn run(&mut self) {
        loop {
            self.poll();
            thread::sleep(Duration::from_millis(50));
        }
    }
}
