use std::net::{TcpStream, SocketAddr};
use std::io::{Read, Write, ErrorKind, Result};
use std::time::{Instant, Duration};

use crate::core::{Request, Response};

#[derive(Debug)]
pub struct ClientConnection {
    pub stream: TcpStream,
    pub peer_addr: SocketAddr,
    pub buffer: Vec<u8>,
    pub timeout: Duration,
    pub last_active: Instant,
}

impl ClientConnection {
    pub fn new(mut stream: TcpStream, peer_addr: SocketAddr, timeout: Duration) -> std::io::Result<Self> {
        stream.set_nonblocking(true)?;
        Ok(Self {
            stream,
            peer_addr,
            buffer: Vec::with_capacity(8192),
            timeout,
            last_active: Instant::now(),
        })
    }

    /// Attempt to read from the stream and append to the buffer.
    pub fn read_into_buffer(&mut self) -> std::io::Result<usize> {
        let mut temp_buf = [0u8; 4096];
        match self.stream.read(&mut temp_buf) {
            Ok(0) => Ok(0), // Connection closed
            Ok(n) => {
                self.buffer.extend_from_slice(&temp_buf[..n]);
                self.last_active = Instant::now();
                Ok(n)
            }
            Err(e) if e.kind() == ErrorKind::WouldBlock => Ok(0), // No data yet
            Err(e) => Err(e),
        }
    }

    pub fn raw_request(&self) -> Option<&str> {
        std::str::from_utf8(&self.buffer).ok()
    }
    
    pub fn parse_request(&self) -> Option<Request> {
        self.raw_request()
            .and_then(|raw| Request::parse(&raw))
    }

    pub fn refresh_activity(&mut self) {
        self.last_active = Instant::now();
    }

    pub fn is_timed_out(&self) -> bool {
        self.last_active.elapsed() > self.timeout
    }
    
    pub fn send_response(&mut self, response: Response) -> Result<()> {
        let bytes = response.to_bytes();
        self.stream.write_all(&bytes)?;
        self.stream.flush()?;
        Ok(())
    }
}
