
## Development Action List
- [✓] **Set up Project Skeleton**
  - [✓] Initialize Cargo project.
  - [✓] Create directory structure (`src/`, `config/`, `core/`, `http/`, `server/`).

- [✓] **Set up simple port use using config.toml**
  - [✓] Create `config/mod.rs` for configuration structs.
  - [✓] Implement basic TOML parsing to read server address and port.
  - [✓] Print out the configured port to verify parsing works.

- [✓] **Set up serverSocket for binding of port**
  - [✓] Create `server/mod.rs` and `server/server.rs`.
  - [✓] Implement socket creation, binding, and listening on the configured port.
  - [✓] Print confirmation of successful binding.

- [✓] **Set up server struct and keep the port open and poll for incoming**
  - [✓] Define `Server` struct to hold server state.
  - [✓] Implement event loop using thread::sleep to simulate polling. Temporary for multi platform compatibility.
  - [✓] Print incoming connection attempts for verification.

- [✓] **Set up connection struct to handle each client connection**
  - [✓] Create `core/connection.rs` to manage individual connections.
  - [✓] Accept incoming connections and instantiate `Connection` objects.
  - [✓] Print raw request for verification.

- [✓] **Set up httpRequest struct and instance one for each client request**
  - [✓] Create `core/request.rs` to parse HTTP requests.
  - [✓] Read raw bytes from connection and parse into `Request` struct.
  - [✓] Print parsed request details for verification.

- [✓] **Construct Basic HTTP Response**
  - [✓] Create a simple response struct in `core/response.rs`.
  - [✓] Send a fixed "Hello World" response.
  - [✓] Test connection handling.

- [✓] **Routing Basic Requests**
  - [✓] Implement simple routing based on URL paths in `server/route.rs`.
  - [✓] Dispatch requests accordingly.

- [✓] **Serve Static Files**
  - [✓] Implement logic in `handler/static.rs` to serve files from a directory.
  - [✓] Map URL paths to file paths.
  - [✓] Send file contents with proper headers.
  - [✓] Test with static assets.

- [✓] **handle http methods**
  - [✓] Implement handling for `GET`, `POST`, and `DELETE` methods in `core/request.rs` and `server/server.rs`.
  - [✓] Validate method support and respond with appropriate status codes for unsupported methods.

- [ ] **Handle CGI Scripts**
  - [ ] Implement CGI script execution in `handler/cgi.rs`.
  - [ ] Map routes to scripts.
  - [ ] Capture output and return as response.
  - [ ] Handle environment variables and permissions.

- [ ] **Implement Basic Routing & Dispatch**
  - [ ] Enhance route matching.
  - [ ] Dispatch to static, CGI, redirect handlers.

- [ ] **Handle Directory Listing and Index Files**
  - [ ] Serve index files automatically.
  - [ ] Generate directory listings if needed.

- [ ] **Add Support for Redirects**
  - [ ] Implement URL redirection in `handler/redirect.rs`.

- [ ] **Implement Graceful Shutdown**
  - [ ] Capture termination signals.
  - [ ] Close server gracefully and finish ongoing requests.

- [ ] **Implement Error Handling**
  - [ ] Standard server error responses (`404`, `500`, etc.).
  - [ ] Use in request flow for errors.

- [ ] **Support for Advanced Features (Optional)**
  - [ ] Add session management, cookies, file uploads, etc.

- [ ] **Testing & Optimization**
  - [ ] Write unit and integration tests.
  - [ ] Optimize routing, file access, concurrency.

---

# HTTP 1.1

HTTP (Hypertext Transfer Protocol) is an application‑level, stateless, request/response protocol for distributed, collaborative, hypermedia systems.

In HTTP/1.1, a connection may be used for one or more request/response exchanges, although connections may be closed for a variety of reasons (see section 8.1).2

## HTTP Message Format
An HTTP message consists of:
- A start-line
- Zero or more header fields (also known as "headers"), each consisting of a name followed by a colon (":") and the field value
- An empty line (i.e., a line with nothing preceding the CRLF) indicating the end of the header fields
- An optional message body

### Request Message Format
An HTTP request message from a client to a server includes:
```
<method> <request-target> <HTTP-version>CRLF
<Header-Name>: <value>CRLF
CRLF
<optional body>
```

```
POST /index.html HTTP/1.1CRLF
Host: example.comCRLF
Content-Length: 13
CRLF
name=John+Doe
```

### Response Message Format
An HTTP response message from a server to a client includes:
```
<HTTP-version> <status-code> <reason-phrase>CRLF
<Header-Name>: <value>CRLF
CRLF
<optional body>
```

```
HTTP/1.1 200 OKCRLF
Content-Type: text/htmlCRLF
Content-Length: 20CRLF
CRLF
<h1>Hello, World!</h1>
```

## Methods
HTTP defines a set of request methods to indicate the desired action to be performed for a given resource. The most commonly used methods are:
- `GET`: Requests a representation of the specified resource. Requests using GET should only retrieve data.
- `POST`: Submits data to be processed to a specified resource.
- `PUT`: Uploads a representation of the specified resource.
- `DELETE`: Deletes the specified resource.
- `HEAD`: Asks for a response identical to that of a GET request, but without the response body.
- `OPTIONS`: Describes the communication options for the target resource.
- `PATCH`: Applies partial modifications to a resource.
- `TRACE`: Performs a message loop-back test along the path to the target resource.
- `CONNECT`: Establishes a tunnel to the server identified by the target resource.

## Status Codes
HTTP response status codes indicate whether a specific HTTP request has been successfully completed. Responses are grouped in five classes:
- 1xx (Informational): The request was received, continuing process.
- 2xx (Successful): The request was successfully received, understood, and accepted.
- 3xx (Redirection): Further action needs to be taken in order to complete the request.
- 4xx (Client Error): The request contains bad syntax or cannot be fulfilled.
- 5xx (Server Error): The server failed to fulfill an apparently valid request.
Common status codes include:
- 200 OK: The request has succeeded.
- 201 Created: The request has been fulfilled and resulted in a new resource being created.
- 204 No Content: The server successfully processed the request, but is not returning any content.
- 301 Moved Permanently: The requested resource has been assigned a new permanent URI.
- 302 Found: The requested resource resides temporarily under a different URI.
- 400 Bad Request: The server could not understand the request due to invalid syntax.
- 401 Unauthorized: The request requires user authentication.
- 403 Forbidden: The server understood the request, but refuses to authorize it.
- 404 Not Found: The server has not found anything matching the Request-URI.
- 500 Internal Server Error: The server encountered an unexpected condition that prevented it from fulfilling the request.
- 502 Bad Gateway: The server, while acting as a gateway or proxy, received an invalid response from the upstream server.
- 503 Service Unavailable: The server is currently unable to handle the request due to temporary overloading or maintenance of the server.
- 504 Gateway Timeout: The server, while acting as a gateway or proxy, did not receive a timely response from the upstream server.

## Headers
HTTP headers allow the client and the server to pass additional information with the request or the response. Common headers include:
- Content-Type: Indicates the media type of the resource.
- Content-Length: Indicates the size of the entity-body, in bytes, sent to the recipient.
- User-Agent: Contains information about the user agent originating the request.
- Accept: Indicates the media types that are acceptable for the response.
- Host: Specifies the domain name of the server and (optionally) the TCP port number
- Authorization: Contains the credentials to authenticate a user-agent with a server.
- Cookie: Contains stored HTTP cookies previously sent by the server with the Set-Cookie header.
- Set-Cookie: Sends cookies from the server to the user agent.
- Cache-Control: Directives for caching mechanisms in both requests and responses.
- Connection: Controls whether the network connection stays open after the current transaction finishes.
- Referer: The address of the previous web page from which a link to the currently requested page was followed.
- Location: Used in redirection or when a new resource has been created.
- ETag: Provides the current value of the entity tag for the requested variant.
- Last-Modified: Indicates the date and time at which the origin server believes the resource was last modified.
- If-Modified-Since: Makes the request conditional: the server will send back the resource only if it has been modified since the specified date.
- If-None-Match: Makes the request conditional: the server will send back the resource only if the entity tag does not match any of the listed tags.
- Vary: Indicates the set of request headers that determine whether a cached response can be used rather than requesting a fresh one from the origin server.
- Transfer-Encoding: Specifies the form of encoding used to safely transfer the entity to the user.
- Accept-Encoding: Indicates the content-codings that are acceptable in the response.
- Accept-Language: Indicates the natural languages that are preferred in the response.
- Origin: Indicates where a fetch originates from.
- Access-Control-Allow-Origin: Specifies which origins are permitted to read the response.
- Access-Control-Allow-Methods: Specifies the methods allowed when accessing the resource in response to a preflight request.
- Access-Control-Allow-Headers: Used in response to a preflight request to indicate which HTTP headers can be used during the actual request.
- Access-Control-Max-Age: Indicates how long the results of a preflight request can be cached.
- Content-Encoding: Used to specify any additional content encodings that have been applied to the entity
- Content-Language: Describes the natural language(s) of the intended audience for the enclosed entity.
- Content-Disposition: Indicates if the content is expected to be displayed inline in the browser, or as an attachment that is downloaded and saved locally.
- Retry-After: Indicates how long the user agent should wait before making a follow-up request.
- Pragma: Implementation-specific headers that may have various effects anywhere along the request-response chain.
- Server: Contains information about the software used by the origin server to handle the request.
- Date: Represents the date and time at which the message was originated.
- Age: The time in seconds the object has been in a proxy cache.
- Via: Informs the client of proxies through which the response was sent.
- Warning: A general warning about possible problems with the entity body.
- TE: Indicates what extension transfer-codings it is willing to accept in the response.

Headers are case-insensitive and can be extended with custom headers as needed.
Header values can be single or multiple, with multiple values separated by commas.

Linear white space(LWS) is treated as a single space.
The Host header field must be sent in all HTTP/1.1 request messages.
If missing or invalid, server should response with a 400 (Bad Request) status code.


## Versioning
HTTP/1.1 is an improvement over HTTP/1.0, introducing several new features and enhancements, including:
- Persistent connections: Allows multiple requests and responses to be sent over a single connection, reducing latency
- Chunked transfer encoding: Enables the server to send data in chunks, allowing it to start sending a response before knowing its total size
- Additional cache control mechanisms: Provides more granular control over caching behavior
- More status codes: Introduces new status codes to better represent various response scenarios
- Host header: Requires the Host header in requests, allowing multiple domains to be served from a single IP address
- Content negotiation: Allows clients and servers to negotiate the best representation of a resource based on factors like language, encoding, and media type
- Enhanced error handling: Provides more detailed error messages and status codes

## Persistent Connections
By default, HTTP/1.1 uses persistent connections, meaning that the connection is kept open for multiple requests and responses, reducing latency for subsequent requests. The `Connection` header can be used to manage the persistence of the connection:
- `Connection: keep-alive`: Indicates that the connection should be kept open for further requests
- `Connection: close`: Indicates that the connection should be closed after the current request/response

## Chunked Transfer Encoding
HTTP/1.1 supports chunked transfer encoding, which allows a server to send a response in chunks, enabling it to start sending data before knowing the total size of the response. This is indicated by the `Transfer-Encoding: chunked` header. Each chunk is preceded by its size in bytes (in hexadecimal), followed by a CRLF, the chunk data, and another CRLF. The end of the response is indicated by a chunk of size zero.
```
HTTP/1.1 200 OKCRLF
Content-Type: text/plainCRLF
Transfer-Encoding: chunkedCRLF
CRLF
4CRLF
WikiCRLF
5CRLF
pediaCRLF
ECRLF
 inCRLF
chunks.CRLF
0CRLF
CRLF
```

## Caching
HTTP/1.1 includes several headers to control caching behavior, allowing clients and servers to specify how responses should be cached and for how long. Common caching headers include:
- Cache-Control: Directives for caching mechanisms in both requests and responses (e.g., `no-cache`, `no-store`, `max-age`, `public`, `private`).
- Expires: Specifies the date/time after which the response is considered stale.
- ETag: Provides a unique identifier for a specific version of a resource, allowing clients to make conditional requests.
- Last-Modified: Indicates the last modification date of the resource, allowing clients to make conditional requests based on this date.
- Vary: Indicates the set of request headers that determine whether a cached response can be used rather than requesting a fresh one from the origin server.

## Security
HTTP/1.1 does not include built-in security features. However, it can be used in conjunction with TLS (Transport Layer Security) to provide secure communication over the network. When HTTP is used over TLS, it is referred to as HTTPS (HTTP Secure). HTTPS ensures data integrity, confidentiality, and authentication between the client and server.

## References
- [RFC 2616 - Hypertext Transfer Protocol -- HTTP/1.1](https://datatracker.ietf.org/doc/html/rfc2616)
- [MDN Web Docs - HTTP](https://developer.mozilla.org/en-US/docs/Web/HTTP)
- [W3C - HTTP/1.1](https://www.w3.org/Protocols/rfc2616/rfc2616.html)
- [IETF - HTTP/1.1](https://www.ietf.org/rfc/rfc2616.txt)
- [Wikipedia - HTTP](https://en.wikipedia.org/wiki/HTTP)
- [HTTP Status Codes](https://httpstatuses.com/)
- [HTTP Headers](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers)
- [Understanding HTTP](https://www.tutorialspoint.com/http/index.htm)
- [HTTP/1.1 Specification](https://tools.ietf.org/html/rfc2616)
- [HTTP/1.1 vs HTTP/2](https://www.keycdn.com/blog/http2-vs-http1)
- [HTTP/1.1 Persistent Connections](https://www.w3.org/Protocols/rfc2616/rfc2616-sec8.html#sec8.1)
- [Chunked Transfer Encoding](https://www.w3.org/Protocols/rfc2616/rfc2616-sec3.html#sec3.6.1)
- [Caching in HTTP](https://developer.mozilla.org/en-US/docs/Web/HTTP/Caching)
- [Security Considerations for HTTP](https://www.ietf.org/rfc/rfc7231.html#section-8.8)
- [Using TLS with HTTP](https://tools.ietf.org/html/rfc2818)
- [HTTP/1.1 Message Syntax and Routing](https://www.w3.org/Protocols/rfc2616/rfc2616-sec3.html)
- [HTTP/1.1 Semantics and Content](https://www.w3.org/Protocols/rfc2616/rfc2616-sec9.html)
- [HTTP/1.1 Conditional Requests](https://www.w3.org/Protocols/rfc2616/rfc2616-sec14.html#sec14.25)
- [HTTP/1.1 Range Requests](https://www.w3.org/Protocols/rfc2616/rfc2616-sec14.html#sec14.35)
- [HTTP/1.1 Content Negotiation](https://www.w3.org/Protocols/rfc2616/rfc2616-sec12.html)
- [HTTP/1.1 Proxy and Tunneling](https://www.w3.org/Protocols/rfc2616/rfc2616-sec13.html)
- [HTTP/1.1 Upgrade Mechanism](https://www.w3.org/Protocols/rfc2616/rfc2616-sec14.html#sec14.42)
- [HTTP/1.1 Internationalization](https://www.w3.org/Protocols/rfc2616/rfc2616-sec3.html#sec3.4)
- [HTTP/1.1 Performance Considerations](https://www.w3.org/Protocols/rfc2616/rfc2616-sec8.html#sec8.1.4)
- [HTTP/1.1 Error Handling](https://www.w3.org/Protocols/rfc2616/rfc2616-sec10.html)
- [HTTP/1.1 Best Practices](https://www.oreilly.com/library/view/high-performance-web/9781449382610/ch04.html)
- [HTTP/1.1 Troubleshooting](https://www.keycdn.com/support/http-troubleshooting)
