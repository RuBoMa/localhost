# Localhost - Event-Driven HTTP Server

A high-performance HTTP/1.1 server built with Rust, featuring event-driven I/O using `kqueue` (macOS/BSD), non-blocking connections, multipart file uploads, and virtual host support.

## Features

**Event-driven architecture** - Non-blocking I/O with kernel-level `kqueue` event notification  
**Concurrent connections** - Efficiently handles hundreds of simultaneous clients  
**HTTP/1.1 persistent connections** - Keep-alive support with configurable timeouts  
**Static file serving** - Efficient file delivery from configured root directories  
**Multipart form uploads** - RFC 2388 compliant file upload handling  
**Flexible routing** - File serving, directory listing, redirects, and upload handlers  
**TOML configuration** - Simple, readable server configuration  
**Connection cleanup** - Proper deregistration of closed sockets

## Quick Start

### Prerequisites

- Rust 1.70+ (macOS/BSD)
- Cargo

### Build

```bash
cd localhost
cargo build --release
```

### Run

```bash
cargo run --release
```

Expected output:

```
[+] Bound to localhost:8080
[+] Bound to localhost:8081
[*] Server initialized
```

### Test

```bash
# Simple request
curl http://localhost:8080/

# Upload file
curl -F "file=@test.txt" http://localhost:8080/upload
```

## CI Pipeline (GitHub Actions)

This repository now includes an automated CI pipeline in `.github/workflows/ci.yml`.

### What Runs in CI

CI starts automatically on:

- push to `main`
- pull requests targeting `main`

It runs two jobs on `macos-latest`:

1. **Rust quality checks**
    - `cargo fmt --all -- --check` (advisory)
    - `cargo clippy --all-targets --all-features` (advisory)
    - `cargo test --all --all-features -- --nocapture` (required)
    - `cargo build --release` (required)

2. **HTTP integration tests**
    - Frees ports `8080` and `8081`
    - Starts the server
    - Waits for port readiness (`nc -z 127.0.0.1 8080`)
    - Runs `scripts/ci_integration.sh`
    - Uploads `server.log` as an artifact on every run

### Integration Checks Covered

`scripts/ci_integration.sh` validates:

- `GET /` on `localhost:8080` returns `302` and points to `/login`
- `GET /login` on `localhost:8080` returns `200`
- `GET /this-does-not-exist` on `localhost:8080` returns `302`
- `GET /hello` on `public:8081` returns `200` and CGI response body content
- `GET /this-does-not-exist` on `public:8081` returns `404`

All `curl` calls in CI use connection and total time limits to avoid hanging builds.

### What You Need to Enable on GitHub

Usually nothing extra is required beyond pushing the workflow file, but verify:

1. **Actions are enabled**
    - Repository → **Settings** → **Actions** → **General**
    - Allow GitHub Actions for this repository

2. **Workflow permissions are allowed**
    - Keep default read permissions (the workflow only needs `contents: read`)

3. **(Recommended) Protect `main`**
    - Repository → **Settings** → **Branches** → Add rule for `main`
    - Require status checks before merge
    - Select both CI jobs as required checks

### Next Steps

1. Commit and push these files:
    - `.github/workflows/ci.yml`
    - `scripts/ci_integration.sh`
    - `README.md`
2. Open the **Actions** tab and confirm the `CI` workflow runs.
3. If a run fails, download the `server-log` artifact and inspect the failing step.
4. After CI is stable, make `main` protection rules required.

### Optional Improvements

- Make `fmt` and `clippy` blocking after cleanup of current warnings/style drift.
- Add nightly stress/memory workflow based on `MEMORY_TESTING.md`.
- Add badge to this README:

```markdown
![CI](https://github.com/<owner>/<repo>/actions/workflows/ci.yml/badge.svg)
```

## Configuration

Edit `config/config.toml`:

```toml
[[servers]]
server_address = "localhost"
ports = [8080, 8081]
server_name = "localhost"
root = "./routes"
client_timeout_secs = 30

[[servers.routes]]
path = "/"
type = "file"
file = "index.html"

[[servers.routes]]
path = "/upload"
type = "upload_dir"
upload_dir = "uploads"
```

### Configuration Options

| Option                | Description                           | Default     |
| --------------------- | ------------------------------------- | ----------- |
| `server_address`      | Bind address                          | `localhost` |
| `ports`               | Listen ports (array)                  | `[8080]`    |
| `server_name`         | Virtual host name (Host header match) | `localhost` |
| `root`                | Document root directory               | `./routes`  |
| `client_timeout_secs` | Idle connection timeout               | `30`        |

## Usage Examples and Features

### Static File Serving

```bash
curl http://localhost:8080/
curl http://localhost:8080/index.html
curl http://localhost:8080/style.css
```

### File Upload

```bash
# Single file
curl -F "file=@document.pdf" http://localhost:8080/upload

# Image upload
curl -F "file=@image.jpg" http://localhost:8080/upload

# With custom field name
curl -F "photo=@picture.png" http://localhost:8080/upload
```

Files are saved to the configured `upload_dir` (default: `./uploads`).

### Redirects

```toml
[[servers.routes]]
path = "/old-page"
type = "redirect"
redirect_to = "/new-page"
redirect_code = 301  # 301, 302, 307, 308
```

### Directory Listing

```toml
[[servers.routes]]
path = "/files"
type = "file"
file = "files/"  # Trailing slash for directory listing
```

### CGI Support

```toml
[servers.cgi_handlers]
".py" = "python3"   # Map .py files to Python 3 interpreter
".pl" = "perl"      # Optional: map .pl files to Perl
```

The server can execute CGI scripts based on file extension configuration. This allows dynamic content generation (for example, using `.py` scripts) alongside static file serving without changing existing route behavior. The program will read the script stdoutput, interpret it, and return it to the client as a response.

#### Example Usage

```bash
curl --resolve public:8081:127.0.0.1 \
     -X POST \
     -H "Content-Type: application/json" \
     -d '{"animal":"Cat","age":3}' \
     http://public:8081/hello
```

- Note that the server validates whether or not the `Host` and `Port` sections of the URL (Host:Port) match the configured `server_name` and `port`.

### Custom Error Pages

`Localhost` supports **custom HTTP error pages** for any status code you wish to override. This allows you to serve user-friendly HTML pages instead of generic server responses.

#### How It Works

- Place custom error pages under the server's `root/errors/` directory.
- Map status codes to filenames in the configuration:

```toml
[servers.errors."404"]
filename = "not_found.html"

[servers.errors."500"]
filename = "server_error.html"

[servers.errors."405"]
filename = "method_not_allowed.html"
```

- The server automatically serves these pages when the corresponding HTTP error occurs.
- If a custom page is missing, the server falls back to a minimal default HTML page.

#### Example File Structure

```
root/
├── errors/
│   ├── not_found.html
│   ├── server_error.html
│   └── method_not_allowed.html
├── index.html
└── ...
```

## Architecture

### Event Loop Model

The server uses `kqueue` (kernel event queue) for efficient async I/O:

```
┌─────────────────────────────────────────┐
│  Main Event Loop                        │
│                                         │
│  1. Block on kevent() until events      │
│  2. Process triggered events            │
│  3. Return to step 1                    │
└─────────────────────────────────────────┘
         ↓           ↓           ↓
    Accept      Read           EOF
    client      data          from client
         ↓           ↓           ↓
    Create      Parse      Deregister
    connection  & route     socket
```

### Request Handling Flow

```
Raw TCP bytes
    ↓
┌─────────────────────┐
│ Connection Buffer   │  (4096-byte chunks)
└─────────────────────┘
    ↓
┌─────────────────────┐
│ Request Parser      │  (splits headers/body)
└─────────────────────┘
    ↓
┌─────────────────────┐
│ Resolve Config      │  (matches Host header)
└─────────────────────┘
    ↓
┌─────────────────────┐
│ Route Dispatcher    │  (static/upload/redirect)
└─────────────────────┘
    ↓
┌─────────────────────┐
│ Response Sender     │  (write to socket)
└─────────────────────┘
    ↓
TCP response to client
```

### Why `kqueue`?

`kqueue` is a scalable kernel event notification system (macOS/BSD):

- **Efficient**: No polling needed; kernel wakes server only when events occur
- **Scalable**: O(1) to monitor thousands of file descriptors
- **Low-level control**: Fine-grained event filtering (read, write, errors)

Alternative on Linux: `epoll`  
Alternative on Windows: `IOCP`

### Unsafe FFI Justification

(Foreign Function Interface)
This codebase uses `unsafe` blocks **only** for C FFI to `libc`:

```rust
// Example: Creating kernel event queue
unsafe { kqueue() }  // Returns fd, or -1 on error

// Example: Registering event with kernel
unsafe { kevent(...) }  // Kernel call; must pass C-compatible pointers
```

**Safety guarantee**: No memory unsafety in safe Rust code. Unsafe blocks are minimal, auditable, and necessary for OS interaction.

---

Server responses:

- **413 Payload Too Large**: If file exceeds limit
- **400 Bad Request**: If multipart parsing fails
- **200 OK**: If upload succeeds

### Supported Upload Methods

**Single file (curl):**

```bash
curl -F "file=@myfile.txt" http://localhost:8080/upload
```

### File Storage

Uploaded files are saved to the configured directory with original filenames:

```
uploads/
  ├─ document.pdf
  ├─ image.jpg
  └─ archive.zip
```

---

## Memory Leak Testing

The server is designed for stable memory usage under load. Test it:

### Quick Test (Recommended)

```bash
# Terminal 1: Start server
cargo run --release

# Terminal 2: Monitor memory
top -pid $(pgrep localhost)

# Terminal 3: Run load test
siege -c50 -t30S http://localhost:8080/
```

**Expected:** Memory spikes during load, returns to baseline after.

### Full Testing Guide

See **[MEMORY_TESTING.md](./MEMORY_TESTING.md)** for 7 methods including:

- Real-time `top` monitoring
- `watch` command loops
- Siege load profiles
- Manual curl testing
- Upload-specific testing
- Connection lifecycle testing
- macOS Instruments profiling

---

## Performance Optimization

### Load Testing

**Light test (initial check):**

```bash
siege -c10 -t15S http://localhost:8080/
```

**Medium load (realistic):**

```bash
siege -c50 -t30S http://localhost:8080/
```

**Heavy benchmark:**

```bash
siege -c100 -t60S http://localhost:8080/
```

**Aggressive (stress test):**

```bash
siege -b http://localhost:8080/  # Press Ctrl+C to stop
```

---

## Project Structure

```
localhost/
├── Cargo.toml                 # Rust dependencies and metadata
├── Cargo.lock                 # Locked dependency versions
├── config/
│   └── config.toml           # Server configuration
├── routes/
│   ├── index.html            # Default page
│   └── uploads/              # Upload destination
├── src/
│   ├── lib.rs                # Library root
│   ├── main.rs               # Binary entry point
│   ├── config/
│   │   └── mod.rs            # Config loading and validation
│   ├── core/
│   │   ├── mod.rs            # Core types
│   │   ├── connection.rs      # TCP connection wrapper
│   │   ├── request.rs         # HTTP request parsing
│   │   ├── response.rs        # HTTP response builder
│   │   ├── multipart.rs       # Multipart form parsing
│   │   └── utils.rs           # Utility functions
│   ├── http/
│   │   ├── mod.rs            # HTTP types
│   │   ├── cookies.rs         # Cookie handling
│   │   ├── session.rs         # Session management
│   │   └── upload.rs          # Upload handling
│   └── server/
│       ├── mod.rs            # Server types
│       ├── server.rs          # Main server logic
│       ├── event_loop.rs      # kqueue event loop
│       ├── server_socket.rs   # Socket binding
│       ├── route.rs           # Route definitions
│       ├── error.rs           # Error types
│       ├── default_html.rs    # Default error pages
│       └── handler/
│           ├── mod.rs         # Route handler types
│           ├── static_handler.rs  # Static file serving
│           ├── cgi.rs         # CGI execution
│           ├── directory.rs    # Directory listing
│           └── redirect.rs     # HTTP redirects
└── target/                    # Build artifacts
```

---

## Dependencies

Key crates used:

- **`libc`** - C FFI for kqueue/kevent
- **`toml`** - TOML configuration parsing
- **`serde`** - Serialization framework

See `Cargo.toml` for complete list.

---

## References

- **HTTP/1.1 Specification**: RFC 7230-7235
- **Multipart Form Data**: RFC 2388
- **kqueue Manual**: `man kqueue` on macOS/BSD
- **Rust async patterns**: https://tokio.rs/tokio/tutorial/select
- **Event-driven architecture**: https://www.ably.io/topic/event-driven-architecture

---

## Support

For issues, questions, or feedback:

1. Check [MEMORY_TESTING.md](./MEMORY_TESTING.md) for diagnostics
2. Review server logs in terminal output
3. Test with: `curl -v http://localhost:8080/`
4. Monitor with: `top -pid $(pgrep localhost)`

---

**Last Updated:** October 27, 2025  
**Server Version:** 1.0  
**Platform:** macOS/BSD with `kqueue`
