# cvkg-webkit-server

**cvkg-webkit-server** provides a development server with WebSocket support for CVKG applications.

## What This Crate Does

- Serves static assets for CVKG applications
- Provides WebSocket endpoint for hot-reload
- Implements basic HTTP server with CORS support
- Monitors file changes for automatic rebuild

## What This Crate Does NOT Do

- Does not provide production-grade hosting
- Does not handle HTTPS termination
- Does not provide authentication or authorization

## Public API Overview

### Server

```rust
/// Development server for CVKG applications
pub struct Server {
    addr: SocketAddr,
    watcher: Option<RecommendedWatcher>,
}

impl Server {
    /// Create a new server bound to the given address
    pub fn new(addr: SocketAddr) -> Self;
    
    /// Start the server and serve content from the given directory
    pub fn serve(&self, root: PathBuf) -> Result<(), ServerError>;
    
    /// Enable hot-reload for the given file paths
    pub fn watch(&mut self, paths: Vec<PathBuf>);
}```

### Functions

```rust
/// Convenience function to start a development server
pub fn serve(addr: SocketAddr, root: PathBuf, watch: Vec<PathBuf>);
```

## Usage Example

```rust
use cvkg_webkit_server::{Server, serve};
use std::net::SocketAddr;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    serve(
        "127.0.0.1:8080".parse()?
        PathBuf::from("./dist"),
        vec![PathBuf::from("./src")],
    )
}
```

## Feature Flags

| Flag | Default | Description |
|------|---------|-------------|
| `std` | true | Use standard library |
| `tls` | false | Enable HTTPS support |

## Known Limitations

- Single-threaded event loop may block under heavy load
- WebSocket keep-alive is not implemented; connections may timeout
- File watching uses polling on some platforms