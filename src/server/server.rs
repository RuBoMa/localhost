use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::net::SocketAddr;
use std::io;
use std::time::Duration;

use crate::Config;
use crate::config::{ServerConfig, RouteConfig};
use crate::ClientConnection;
use crate::core::{Response, Request};
use crate::server::default_html::{
    default_400_response,
    default_403_response,
    default_404_response,
    default_405_response,
    default_500_response,
    default_index_response,
};
use crate::server::match_route;
use crate::server::handler::{Admin, serve_static_file, generate_directory_listing, resolve_target_path};
use crate::server::ServerSocket;
use crate::server::run_loop;

#[derive(Debug)]
pub struct Server {
    pub sockets: Vec<ServerSocket>,
    pub clients: Vec<ClientConnection>,
    pub client_timeout: Duration,
    pub admin: Admin,
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

        let client_timeout = Duration::from_secs(config.client_timeout_secs);
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

        let admin = Admin::new(config.admin.clone());

        Ok(Server {
            sockets,
            clients: Vec::new(),
            client_timeout,
            admin,
        })
    }

    pub fn handle_directory_request(
        &self,
        root_dir: &Path,
        route_cfg: &RouteConfig,
        request_subpath: &str,
        route_prefix: &str,
    ) -> Response {
        // Full target path is base_path + request_subpath
        let target_path = root_dir.join(request_subpath.trim_start_matches('/'));
        let Ok(target_path) = target_path.canonicalize() else {
            return default_404_response();
        };
        let Ok(root_dir) = root_dir.canonicalize() else {
            return default_500_response(Some("Invalid root directory"));
        };
        if !target_path.starts_with(&root_dir) {
            return default_403_response();
        }

        if target_path.is_file() {
            return serve_static_file(&target_path);
        }

        if target_path.is_dir() {
            if route_cfg.directory_listing {
                return generate_directory_listing(
                    &target_path,
                    route_prefix,
                    route_cfg.upload_dir.is_some()
                );
            }

            if let Some(file) = route_cfg.filename.clone() {
                return serve_static_file(&target_path.join(file));
            }

            if target_path.join("index.html").exists() {
                return serve_static_file(&target_path.join("index.html"));
            }
            return default_403_response();
        }

        default_404_response()
    }

    fn handle_request(&mut self, request: &Request, client: &ClientConnection) -> Response {
        // Step 1: Identify which socket the client connected to
        let socket = match self.sockets.iter().find(|s| s.addr == client.local_addr) {
            Some(sock) => sock,
            None => {
                eprintln!("[!] No socket found for local addr {}", client.local_addr);
                return default_500_response(Some("Socket not found."));
            }
        };

        // Step 2: Resolve configuration based on Host header
        let host_header = request.headers.get("Host").map(|s| s.as_str());
        let config = socket.resolve_config(host_header);
        let root_dir = Path::new(&config.root);

        // ✅ Step 2.5: Check admin access requirement
        if socket.requires_admin_auth() {
            let is_authenticated = self.admin.validate_request(request);

            // If not authenticated
            if !is_authenticated {
                if request.uri == "/login" {
                    if request.method.eq_ignore_ascii_case("POST") {
                        // POST → attempt login
                        return self.admin.handle_login(request);
                    } else {
                        // GET → serve login page
                        return serve_static_file(&root_dir.join("login.html"));
                    }
                } else {
                    // Any other admin route → redirect to login
                    return Response::redirect("/login".to_string(), 302);
                }
            }
        }

        // Step 3: Find route match
        let (route_prefix, route_cfg) = match match_route(&config.routes, &request.uri) {
            Some(r) => r,
            None => return default_404_response(),
        };

        // Step 4: Redirect
        if let Some(redirect) = &route_cfg.redirect {
            return Response::redirect(redirect.to.clone(), redirect.code);
        }

        // Step 5: Enforce allowed methods
        if let Err(allowed_methods) = route_cfg.check_method(&request.method) {
            return default_405_response(Some(&allowed_methods));
        }

        // Step 6: Upload handling (POST → upload_dir)
        if let Some(upload_dir) = &route_cfg.upload_dir {    
            let full_target_path = resolve_target_path(
                &request.uri,
                &route_prefix,
                root_dir, &upload_dir);

            if request.method.eq_ignore_ascii_case("POST") {
                return Server::handle_upload(request, &full_target_path);
            }
            
            if request.method.eq_ignore_ascii_case("DELETE") {
                return Server::handle_delete(&full_target_path);
            }
        }
        
        // Step 7: Directory handling (GET/HEAD → directory or listing)
        if let Some(dir) = &route_cfg.directory {
            if !matches!(request.method.as_str(), "GET" | "HEAD") {
                return default_405_response(Some("GET, HEAD"));
            }
            let base_dir = root_dir.join(dir);
            let sub_path = &request.uri[route_prefix.len()..];
            let sub_path = if sub_path.is_empty() { "/" } else { sub_path };
            let route_prefix = format!("{}/{}", route_prefix.trim_end_matches('/'), sub_path.trim_start_matches('/'));

            return self.handle_directory_request(&base_dir, route_cfg, sub_path, &route_prefix);
        }

        // Step 8: Static file handling (GET/HEAD → filename)
        if let Some(filename) = &route_cfg.filename {
            if !matches!(request.method.as_str(), "GET" | "HEAD") {
                return default_405_response(Some("GET, HEAD"));
            }

            let full_path = root_dir.join(filename);
            return serve_static_file(&full_path);
        }

        // Step 9: Misconfiguration. serve default index
        default_index_response(&config.routes)
    }

    pub fn handle_client(&mut self, client: &mut ClientConnection) -> io::Result<bool> {
        match client.read_into_buffer() {
            Ok(0) => {
                println!("[*] Client {} closed the connection", client.peer_addr);
                return Ok(false); // Tcp will close on drop
            }
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
            return default_500_response(
                Some(&format!("Could not create upload directory: {}", e))
            );
        }

        if !request.is_multipart() {
            return default_400_response();
        }

        let parts = match request.multipart_parts() {
            Some(p) => p,
            None => return default_400_response()
        };

        for part in parts {

            if let Some(filename) = &part.filename {
                // Build full path under upload_directory
                let full_path = Path::new(upload_directory).join(filename);

                if let Some(parent) = full_path.parent() {
                    if let Err(e) = std::fs::create_dir_all(parent) {
                        eprintln!("❌ Failed to create directories for {}: {}", full_path.display(), e);
                        continue;
                    }
                }
                
                match std::fs::write(&full_path, &part.content) {
                    Ok(_) => println!("✅ Saved file: {}", full_path.display()),
                    Err(e) => eprintln!("❌ Failed to save {}: {}", full_path.display(), e),
                }
            }
        }

        Response::new(200, "OK").with_body("File uploaded successfully\n")
    }

    pub fn handle_delete(target_path: &Path) -> Response {
        if !target_path.exists() {
            return default_404_response();
        }

        let result = if target_path.is_file() {
            std::fs::remove_file(target_path)
        } else if target_path.is_dir() {
            std::fs::remove_dir_all(target_path)
        } else {
            Err(std::io::Error::new(std::io::ErrorKind::Other, "Unknown file type"))
        };

        match result {
            Ok(_) => Response::new(200, "OK")
                .with_body(format!("Deleted successfully: {}", target_path.display())),
            Err(e) => default_500_response(
                Some(&format!("Failed to delete {}: {}", target_path.display(), e))
            ),
        }
    }

    pub fn run(&mut self) {
        run_loop(self);
    }
}
