# Simple HTTP Server in Rust

I am build a simple HTTP 1.1 server written in Rust. My goal is to provide a lightweight, efficient, and easy-to-use server for serving static files, handling CGI scripts, and managing basic web applications.
To understand the logic better, we will not use external crates.
We will only use `epoll` for event polling and `toml` for configuration parsing.
We will use one process and one thread to handle multiple connections concurrently using non-blocking I/O and event-driven programming.

currently I am working on windows wusing thread::sleep to simulate polling. Temporary for multi platform compatibility.
We might setup the server to be run in a debian docker container for epoll support later.

## Project Structure
The project is organized as follows:
```
localhost/
├── Cargo.toml
├── src/
│   ├── lib.rs                  # Crate root (public API if reused)
│   ├── main.rs                 # Entry point (if building a binary)
│
│   ├── config/                 # Configuration system
│   │   ├── mod.rs              # Global + per-route config structs
│
│   ├── core/                   # HTTP core: parsing, building, streams
│   │   ├── mod.rs
│   │   ├── request.rs
│   │   ├── response.rs
│   │   ├── connection.rs       # Handles one TCP connection
│   │   └── utils.rs            # Common helpers
│
│   ├── http/                   # HTTP protocol features
│   │   ├── mod.rs
│   │   ├── cookies.rs
│   │   ├── session.rs
│   │   ├── upload.rs
│
│   ├── server/                 # Server runtime & logic
│   │   ├── mod.rs
│   │   ├── server.rs           # Multi-port event loop
│   │   ├── route.rs            # Route matcher
│   │   ├── error.rs            # Error responses
│   │   └── handler/            # Request dispatch
│   │       ├── mod.rs
│   │       ├── static.rs
│   │       ├── cgi.rs
│   │       ├── redirect.rs
│   │       └── directory.rs
```

The server must guarantee the following behavior:
- It never crashes.
- All requests timeout if they are taking too long.
- It can listen on multiple ports and instantiate multiple servers at the same time.
- You use only one process and one thread.
- It receives a request from the browser/client and send a response using the HTTP header and body.
- It manages at least [GET, POST, DELETE] methods.
- It is able to receive file uploads made by the client.
- It handles cookies and sessions.
- You should create default error pages for at least the following error codes [400,403,404,405,413,500].
- It calls epoll function (or equivalent) only once for each client/server communication.
- All reads and writes should pass by epoll or equivalent API.
- All I/O operations should be non-blocking.
- You should manage chunked and unchunked requests.
- You should set the right status for each response.

Configuration File should be able to specify the following:
- The host (server_address) and one or multiple ports for each server.
- The first server for a host:port will be the default if the "server_name" didn't match any other server.
- Path to custom error pages.
- Limit client body size for uploads.
- Setup routes with one or multiple of the following settings:
  - Define a list of accepted HTTP methods for the route.
  - Define HTTP redirections.
  - Define a directory or a file from where the file should be searched (for example, if /test is rooted to /usr/Desktop, the URL /test/my_page.html will route to /usr/Desktop/my_page.html).
  - Define a default file for the route if the URL is a directory.
  - Specify a CGI to use for a certain file extension.
  - Turn on or off directory listing.
  - Set a default file to answer if the request is a directory.

Lets take it step by step and learn as we go. I don't want you to spit out everything at once.
I will share some of the code I have.

akcnowledge the prompt and wait for my next instructions.
