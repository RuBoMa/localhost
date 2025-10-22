use std::collections::HashMap;
use std::io::Write;
use std::io::{self, ErrorKind};
use std::net::{SocketAddr, TcpListener};
use std::path::Path;
use std::{thread, time::Duration};

use crate::config::ServerConfig;
use crate::ClientConnection;
use crate::core::{Response, Request};
use crate::server::default_html::{
    default_404_response,
    default_welcome_response,
    default_method_not_allowed_response
};
use crate::server::handler::serve_static_file;
use crate::Config;

use libc::{kevent, kevent64_s, kqueue, EVFILT_READ, EV_ADD, EV_DELETE, EV_ENABLE};
use std::os::fd::AsRawFd;

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

    pub fn timeout(&self, server_name: Option<&str>) -> Duration {
        Duration::from_secs(self.resolve_config(server_name).client_timeout_secs)
    }

    /// Accepts all pending connections (non-blocking), returns new clients.
    pub fn try_accept(&self) -> Vec<ClientConnection> {
        let mut new_clients = Vec::new();

        loop {
            match self.listener.accept() {
                Ok((stream, peer_addr)) => {
                    println!(
                        "[*] Accepted connection from {} on {}",
                        peer_addr, self.addr
                    );
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
        // Step 1: Get the socket the client connected to (by local_addr)
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

        // Step 2: Extract server_name from the Host header
        let host_header = request.headers.get("Host").map(|s| s.as_str());
        let config = socket.resolve_config(host_header);
        let root_dir = Path::new(&config.root);

        // Step 3: Show default welcome page only if root directory doesn't exist
        if !root_dir.exists() {
            return default_welcome_response()
        }

        // Step 4: Route matching
        if let Some(route_cfg) = config.routes.get(&request.uri) {
            // ✅ Step 4.1: Check if method is allowed
            if let Some(allowed_methods) = &route_cfg.methods {
                if !allowed_methods.iter().any(|m| m.eq_ignore_ascii_case(&request.method)) {
                    let allow_header = allowed_methods.join(", ");
                    return default_method_not_allowed_response(Some(&allow_header));
                }
            }

            // ✅ Step 4.2: Handle redirect if defined
            if let Some(redirect) = &route_cfg.redirect {
                return Response::redirect(redirect.to.clone(), redirect.code);
            }

            // ✅ Step 4.3: Serve static file if filename is defined
            if let Some(filename) = &route_cfg.filename {
                let full_path = root_dir.join(filename);
                return serve_static_file(&full_path);
            }

            // ✅ Step 4.4: Misconfigured route (no redirect or filename)
            return Response::new(500, "Internal Server Error")
                .header("Content-Type", "text/html")
                .with_body("<h1>500 Internal Server Error</h1><p>Route is misconfigured (no redirect or file).</p>");
    
        } else {
            // Route not defined in config, but root exists
            default_404_response()
        }
    }

    fn handle_client(&mut self, client: &mut ClientConnection) -> io::Result<bool> {
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

    pub fn run(&mut self) {
        // Create kqueue
        let kqueue = unsafe { kqueue() };
        if kqueue == -1 {
            panic!("Failed to create kqueue");
        }

        // Register listening sockets
        for socket in &self.sockets {
            let fd = socket.listener.as_raw_fd();
            let mut event = kevent64_s {
                ident: fd as u64,          // WHAT to monitor (the socket fd)
                filter: EVFILT_READ,       // WHAT KIND of event (read events)
                flags: EV_ADD | EV_ENABLE, // WHAT ACTION to take (register for read)
                fflags: 0,                 // no filter-specific flags (none needed for EVFILT_READ)
                data: 0,                   // no filter-specific data (none needed for EVFILT_READ)
                udata: 0,                  // USER data (not used here)
                ext: [0, 0],               // EXTENDED data (not used)
            };

            let result = unsafe {
                kevent(
                    kqueue,
                    &mut event as *mut _ as *const _,
                    1,
                    std::ptr::null_mut(),
                    0,
                    std::ptr::null(),
                )
                };
                 if result == -1 {
                    panic!(
                        "[!] Failed to register socket {} with kqueue: {}",
                        fd,
                        std::io::Error::last_os_error()
                    );
                }
            }

        // Prepare event buffer
        let mut events = vec![
            kevent64_s {
                ident: 0,
                filter: 0,
                flags: 0,
                fflags: 0,
                data: 0,
                udata: 0,
                ext: [0, 0],
            };
            1024
        ];

        loop {
            // Wait for events
            let nev = unsafe {
                kevent(
                    kqueue,
                    std::ptr::null(),
                    0,
                    events.as_mut_ptr() as *mut _,
                    events.len() as i32,
                    std::ptr::null(),
                )
            };

            if nev < 0 {
                eprintln!("[!] kqueue wait failed");
                continue;
            }

            // Handle triggered events
            for i in 0..nev as usize {
                let ev = &events[i];
                let fd = ev.ident as i32;

                // Is it a listening socket?
                if let Some(socket) = self.sockets.iter().find(|s| s.listener.as_raw_fd() == fd) {
                    let new_clients = socket.try_accept();
                    for client in new_clients {
                        let cfd = client.stream.as_raw_fd();

                        // Register client socket for READ
                        let mut client_ev = kevent64_s {
                            ident: cfd as u64,
                            filter: EVFILT_READ,
                            flags: EV_ADD | EV_ENABLE,
                            fflags: 0,
                            data: 0,
                            udata: 0,
                            ext: [0, 0],
                        };

                        unsafe {
                            kevent(
                                kqueue,
                                &mut client_ev as *mut _ as *const _,
                                1,
                                std::ptr::null_mut(),
                                0,
                                std::ptr::null(),
                            );
                        }

                        self.clients.push(client);
                    }
                } else {
                    // Existing client
                    if let Some(pos) = self.clients.iter().position(|c| c.stream.as_raw_fd() == fd)
                    {
                        let mut client = self.clients.swap_remove(pos);
                        let keep = self.handle_client(&mut client).unwrap_or(false);

                        if keep {
                            self.clients.push(client);
                        } else {
                            // Deregister closed client
                            let mut ev_del = kevent64_s {
                                ident: fd as u64,
                                filter: EVFILT_READ,
                                flags: EV_DELETE,
                                fflags: 0,
                                data: 0,
                                udata: 0,
                                ext: [0, 0],
                            };
                            unsafe {
                                kevent(
                                    kqueue,
                                    &mut ev_del as *mut _ as *const _,
                                    1,
                                    std::ptr::null_mut(),
                                    0,
                                    std::ptr::null(),
                                );
                            }
                        }
                    }
                }
            }
        }
    }
}
