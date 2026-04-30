# CVKG Server Deployment Guide

This guide describes how to deploy the CVKG WebKit Server to a production environment.

## Prerequisites

- Linux-based OS (Ubuntu 22.04+ recommended)
- Rust 1.75+ (for building from source)
- `pkg-config`, `libssl-dev`, `libwebkit2gtk-4.0-dev` (for build-time dependencies)

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `CVKG_BIND_ADDR` | Address to bind the server to | `0.0.0.0:3000` |
| `CVKG_PKG_DIR` | Directory for WASM artifacts | `cvkg-webkit-server/pkg` |
| `CVKG_ASSETS_DIR` | Directory for static assets | `cvkg-webkit-server/assets` |
| `CVKG_STATIC_DIR` | Directory for static HTML/CSS | `cvkg-webkit-server/static` |
| `CVKG_RATE_LIMIT_RPS` | Rate limit (requests per second) | `100` |
| `CVKG_TIMEOUT_SECS` | Request timeout in seconds | `30` |
| `CVKG_MAX_CONCURRENT` | Max concurrent requests | `100` |
| `RUST_LOG` | Logging level (error, warn, info, debug, trace) | `info` |

## Build and Run

### 1. Build the release binary
```bash
cargo build --release -p cvkg-webkit-server
```

### 2. Prepare the environment
Create a `.env` file or export variables:
```bash
export CVKG_BIND_ADDR=0.0.0.0:80
export RUST_LOG=info
```

### 3. Run the server
```bash
./target/release/cvkg-webkit-server
```

## Production Recommendations

- **Reverse Proxy**: Use Nginx or Caddy in front of the server for TLS termination and additional hardening.
- **Process Manager**: Use `systemd` or `supervisord` to ensure the server stays up.
- **Monitoring**: Scrape the `/metrics` endpoint with Prometheus.
- **Security**: Ensure the firewall only allows ports 80/443.

## Systemd Service Example

```ini
[Unit]
Description=CVKG WebKit Server
After=network.target

[Service]
Type=simple
User=cvkg
Group=cvkg
WorkingDirectory=/opt/cvkg
ExecStart=/opt/cvkg/cvkg-webkit-server
Restart=always
Environment=CVKG_BIND_ADDR=0.0.0.0:3000
Environment=RUST_LOG=info

[Install]
WantedBy=multi-user.target
```
