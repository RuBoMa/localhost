use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::net::SocketAddr;
use std::io;

use crate::config::ServerConfig;
use crate::core::{Request, Response};
use crate::server::default_html::{
    default_404_response, default_method_not_allowed_response, default_welcome_response,
};
use crate::server::handler::serve_static_file;
use crate::server::ServerSocket;
use crate::ClientConnection;
use crate::Config;
use crate::server::run_loop;


#[derive(Debug)]
pub struct Server {
    pub sockets: Vec<ServerSocket>,
    pub clients: Vec<ClientConnection>,
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
                        grouped.entry(addr).or_default().push(server.clone());
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
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "No sockets bound",
            ));
        }

        Ok(Server {
            sockets,
            clients: Vec::new(),
        })
    }

    fn handle_request(&self, request: &Request, client: &ClientConnection) -> Response {
        // Get the socket the client connected to (by local_addr)
        let socket = match self.sockets.iter().find(|s| s.addr == client.local_addr) {
            Some(sock) => sock,
            None => {
                eprintln!("[!] No socket found for local addr {}", client.local_addr);
                return Response::new(500, "Internal Server Error")
                    .header("Content-Type", "text/html")
                    .header("Connection", "close")
                    .with_body("<h1>500 Internal Server Error</h1><p>Socket not found.</p>");
            }
        };

        // Extract server_name from the Host header
        let host_header = request.headers.get("Host").map(|s| s.as_str());
        let config = socket.resolve_config(host_header);
        let root_dir = Path::new(&config.root);

        // Step 3: Show default welcome page only if root directory doesn't exist
        if !root_dir.exists() {
            return default_welcome_response();
        }

        // Route matching
        if let Some(route_cfg) = config.routes.get(&request.uri) {
            // Step 4.1: Check if method is allowed
            if let Some(allowed_methods) = &route_cfg.methods {
                if !allowed_methods
                    .iter()
                    .any(|m| m.eq_ignore_ascii_case(&request.method))
                {
                    let allow_header = allowed_methods.join(", ");
                    return default_method_not_allowed_response(Some(&allow_header));
                }
            }

            // Handle redirect if defined
            if let Some(redirect) = &route_cfg.redirect {
                return Response::redirect(redirect.to.clone(), redirect.code);
            }
            
            // Handle POST /upload specially
            if let Some(upload_dir) = &route_cfg.upload_dir {
                let full_upload_path = root_dir.join(upload_dir);

                // Call your upload handler with the full path
                return Server::handle_upload(request, &full_upload_path);
            }

            // Serve static file if filename is defined
            if let Some(filename) = &route_cfg.filename {
                let full_path = root_dir.join(filename);
                return serve_static_file(&full_path);
            }

            // Misconfigured route (no redirect or filename)
            return Response::new(500, "Internal Server Error")
                .header("Content-Type", "text/html")
                .with_body("<h1>500 Internal Server Error</h1><p>Route is misconfigured (no redirect or file).</p>");
        } else {
            // Route not defined in config, but root exists
            default_404_response()
        }
    }

    pub fn handle_client(&mut self, client: &mut ClientConnection) -> io::Result<bool> {
        match client.read_into_buffer() {
/*             Ok(0) => {
                println!("[*] Client {} closed the connection", client.peer_addr);
                return Ok(false); // Tcp will close on drop
            } */
            Ok(_) => {
                if let Some((request, consumed)) = client.parse_request() {
                    let response = self.handle_request(&request, &client);

                    client.send_response(response)?;
                    client.buffer.drain(..consumed);

                    // Check if client wants to close
                    let close_connection = request
                        .headers
                        .get("Connection")
                        .map(|v| v.eq_ignore_ascii_case("close"))
                        .unwrap_or(false);

                    if close_connection {
                        client.stream.shutdown(std::net::Shutdown::Both)?;
                        return Ok(false); // stop handling
                    }
                }
                // keep connection open for persistent HTTP/1.1
                return Ok(true);
            }
            Err(e) => {
                eprintln!("[!] Error reading from {}: {}", client.peer_addr, e);
                let _ = client.stream.shutdown(std::net::Shutdown::Both);
                Ok(false)
            }
        }
    }

    fn handle_upload(request: &Request, upload_directory: &PathBuf) -> Response {    
        if let Err(e) = std::fs::create_dir_all(upload_directory) {
            return Response::new(500, "Internal Server Error")
                .with_body(format!("Could not create upload directory: {}", e));
        }

        if !request.is_multipart() {
            return Response::new(400, "Bad Request")
                .with_body("Expected multipart/form-data\n");
        }

        let parts = match request.multipart_parts() {
            Some(p) => p,
            None => return Response::new(400, "Bad Request").with_body("Invalid multipart data\n"),
        };

        for part in parts {
            if let Some(filename) = &part.filename {
                // Build full path under upload_directory
                let full_path = Path::new(upload_directory).join(filename);

                match std::fs::write(&full_path, &part.content) {
                    Ok(_) => println!("✅ Saved file: {}", full_path.display()),
                    Err(e) => eprintln!("❌ Failed to save {}: {}", full_path.display(), e),
                }
            }
        }

        Response::new(200, "OK").with_body("File uploaded successfully\n")
    }
 
    pub fn run(&mut self) {
        run_loop(self);
    }
}
