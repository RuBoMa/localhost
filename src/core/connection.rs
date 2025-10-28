use std::io::{ErrorKind, Read, Result, Write};
use std::net::{SocketAddr, TcpStream};
use std::time::Instant;

use crate::core::{Request, Response};

#[derive(Debug)]
pub struct ClientConnection {
    pub stream: TcpStream,
    pub peer_addr: SocketAddr,
    pub local_addr: SocketAddr,
    pub buffer: Vec<u8>,
    pub request_at: Option<Instant>
}

impl ClientConnection {
    pub fn new(stream: TcpStream, peer_addr: SocketAddr) -> std::io::Result<Self> {
        let local_addr = stream.local_addr()?;

        stream.set_nonblocking(true)?;

        Ok(Self {
            stream,
            peer_addr,
            local_addr,
            buffer: Vec::with_capacity(8192),
            request_at: None,
        })
    }

    /// Attempt to read from the stream and append to the buffer.
    pub fn read_into_buffer(&mut self) -> std::io::Result<usize> {
        let mut temp_buf = [0u8; 4096];
        match self.stream.read(&mut temp_buf) {
            Ok(0) => Ok(0), // Connection closed
            Ok(n) => {
                self.buffer.extend_from_slice(&temp_buf[..n]);
                
                if self.request_at.is_none() {
                    self.request_at = Some(Instant::now());
                }
                Ok(n)
            }
            Err(e) if e.kind() == ErrorKind::WouldBlock => Ok(0), // No data yet
            Err(e) => Err(e),
        }
    }
    
    pub fn parse_request(&mut self) -> Option<Request> {
        if let Some((request, consumed)) = Request::parse(&self.buffer) {
            self.buffer.drain(0..consumed);
            self.request_at = None;
            return Some(request);
        } 
        None
    }

    pub fn send_response(&mut self, response: Response) -> Result<()> {
        let bytes = response.to_bytes();
        let mut offset = 0;

        while offset < bytes.len() {
            match self.stream.write(&bytes[offset..]) {
                Ok(0) => break, // connection closed
                Ok(n) => offset += n, // advance by number of bytes written
                Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                    // Kernel buffer full: stop and return for now
                    // You need to wait for writable event (via kqueue/epoll) before continuing
                    break;
                }
                Err(e) => return Err(e),
            }
        }

        Ok(())
    }
}
