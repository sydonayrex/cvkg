# How to Use the CVKG CLI

Goal: Scaffold, audit, compile, and inspect a CVKG application using the unified command-line toolchain.

## Prerequisites
- Rust toolchain (1.85+) installed.
- System dependencies (Vulkan/Metal/DX12 GPU drivers, `libfontconfig1-dev` and `pkg-config` on Linux) installed.

---

## Steps

### 1. Scaffold a New Project
Initialize a fresh application workspace using the default scaffolding template:
```bash
cargo run -p cvkg-cli -- new my_new_app --git
```

### 2. Start the Development Server
Launch the compiler and start the background web server with reactive reload capabilities:
```bash
cargo run -p cvkg-cli -- dev --target wasm --port 3000 --inspector
```

### 3. Run Static Code and Layout Audits
Execute type-checking and verify the workspace status:
```bash
cargo run -p cvkg-cli -- check
```

### 4. Build for Platform Targets
Compile the application for native rendering or web platforms:
```bash
cargo run -p cvkg-cli -- build --target native --release
```

### 5. Launch the Telemetry Inspector
Attach the diagnostic inspector to monitor real-time frame rates, Virtual DOM updates, and video memory consumption:
```bash
cargo run -p cvkg-cli -- inspect --url http://localhost:3000 --ws-port 8081
```

### 6. Export for Web Production
Bundle compile outputs and write a target-ready index file to the production assets folder:
```bash
cargo run -p cvkg-cli -- export --base-path /assets/ --optimize
```

---

## Expected Output
For a successful new project scaffolding, a directory structure is created containing the Cargo manifests, an assets pipeline configuration, and an initial source layout tree. When running the inspector, real-time FPS telemetry is printed directly to the console:
```
📊 FPS: 60 | VRAM: 124 MB | VDOM Diff: 1 ms
```

---

## Recovery and Debugging

### Telemetry Connection Failures
If the telemetry inspector fails to connect to the stream:
1. Verify the development server is active on the expected host port.
2. Confirm no firewall or localhost proxy rules are blocking the WebSocket port.
3. Relaunch with logging output active:
   ```bash
   RUST_LOG=debug cargo run -p cvkg-cli -- inspect --url http://localhost:3000
   ```

### Scaffold target already exists
If the scaffolding script fails because the directory already exists:
1. Move or rename the conflicting directory:
   ```bash
   mv my_new_app my_new_app_backup
   ```
2. Re-run the scaffolding command.
