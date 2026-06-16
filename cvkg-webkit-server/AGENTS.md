# cvkg-webkit-server AGENTS.md

## Purpose
Own the WebKit integration server: embedding WebKit views, bridging between CVKG and web content, and the web-to-native message passing system.

## Ownership
- `src/lib.rs` — WebKit server lifecycle, view management
- JavaScript bridge, message passing protocol
- Security policy enforcement for web content

## Local Contracts
- Web content must be sandboxed with configurable security policies.
- Message passing must be type-safe and serialized correctly.
- Must handle web view lifecycle (load, unload, navigate) gracefully.

## Verification
- Run `cargo test -p cvkg-webkit-server`
- Run `cargo check --workspace`
