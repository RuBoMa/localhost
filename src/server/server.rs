use std::net::{TcpListener, SocketAddr};
use std::io::{self, ErrorKind};
use std::{thread, time::Duration};

use crate::Config;
use crate::ClientConnection;
use crate::core::{Response, Request};

#[derive(Debug)]
pub struct ServerSocket {
    pub addr: SocketAddr,
    pub listener: TcpListener,
    pub client_timeout: Duration,
}

impl ServerSocket {
    pub fn try_bind(addr: SocketAddr, client_timeout: Duration) -> io::Result<Self> {
        let listener = TcpListener::bind(addr)?;
        listener.set_nonblocking(true)?;
        println!("[+] Bound to {}", addr);
        Ok(Self { addr, listener, client_timeout })
    }

    /// Accepts all pending connections (non-blocking), returns any new clients.
    pub fn try_accept(&self) -> Vec<ClientConnection> {
        let mut new_clients = Vec::new();

        loop {
            match self.listener.accept() {
                Ok((stream, peer_addr)) => {
                    println!("[*] Accepted connection from {} on {}", peer_addr, self.addr);
                    match ClientConnection::new(stream, peer_addr, self.client_timeout) {
                        Ok(client) => new_clients.push(client),
                        Err(e) => eprintln!("[!] Failed to create client connection: {}", e),
                    }
                }
                Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                    // No more pending connections
                    break;
                }
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
    pub fn from_config(config: &Config) -> io::Result<Self> {
        let mut sockets = Vec::new();

        for server in &config.servers {
            for &port in &server.ports {
                let addr_str = format!("{}:{}", server.server_address, port);
                match addr_str.parse::<SocketAddr>() {
                    Ok(addr) => match ServerSocket::try_bind(addr, Duration::from_secs(server.client_timeout_secs)) {
                        Ok(socket) => sockets.push(socket),
                        Err(e) => eprintln!("[!] Failed to bind {}: {}", addr, e),
                    },
                    Err(e) => eprintln!("[!] Invalid address '{}': {}", addr_str, e),
                }
            }
        }

        if sockets.is_empty() {
            return Err(io::Error::new(io::ErrorKind::Other, "No sockets bound"));
        }

        Ok(Server {
            sockets,
            clients: Vec::new(),
        })
    }

    fn build_response(&self, request: &Request) -> Response {
        // Very simple placeholder
        Response::new(200, "OK")
            .header("Content-Type", "text/plain")
            .with_body("Hello, world!")
    }

    fn handle_client(&mut self, client: &mut ClientConnection) -> io::Result<bool> {
        if client.is_timed_out() {
            println!("[*] Closing idle client {}", client.peer_addr);
            return Ok(false);
        }

        match client.read_into_buffer() {
            Ok(n) => {
                if n == 0 {
                    println!("[*] Client {} closed the connection", client.peer_addr);
                    return Ok(false);
                }

                client.refresh_activity();

                if let Some(request) = client.parse_request() {
                    println!("--- Parsed Request from {} ---\n{:#?}", client.peer_addr, request);

                    let response = self.build_response(&request);
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