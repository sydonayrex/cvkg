# temp-server

**Temporary server for development and testing CVKG applications**

## Overview

This crate provides a lightweight HTTP server for local development and testing of CVKG web applications. It serves static files and provides endpoints for hot-reloading during development.

## Features

- **Static file serving** for web builds
- **Hot-reload support** for development workflow
- **CORS handling** for local development
- **WebAssembly module loading** for CVKG web apps
- **WebSocket support** for real-time updates

## Usage

```rust
use temp_server::Server;

#[tokio::main]
async fn main() {
    let server = Server::new(