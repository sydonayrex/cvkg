# How to Use the CVKG CLI

## Goal

Use the `cvkg` command-line tool to scaffold projects, start dev servers, and run telemetry.

## Prerequisites

- `cargo build -p cvkg-cli` succeeds
- The `cvkg` binary is available on your PATH

## Steps

### 1. Scaffold a New Project

```bash
cvkg new my-app
```

This creates a new directory `my-app/` with a basic CVKG project structure.

### 2. Start the Dev Server

```bash
cd my-app
cvkg serve
```

The dev server watches for file changes and triggers hot-reload via WebSocket.

### 3. Export Design Tokens

```bash
cvkg export-tokens --format json
```

Exports the current theme's design tokens to stdout or a file.

### 4. Build for Production

```bash
cvkg build --release
```

Compiles the project with optimizations.

### 5. Run Telemetry

```bash
cvkg telemetry
```

Displays performance metrics and accessibility audit results.

## Expected Output

- `cvkg new`: New project directory with Cargo.toml, src/, and assets/.
- `cvkg serve`: Running server with file watcher. Output shows WebSocket URL.
- `cvkg export-tokens`: JSON or TOML token output.
- `cvkg build`: Compiled binary in `target/release/`.

## What Can Go Wrong

- **"command not found"**: Build the CLI first: `cargo build -p cvkg-cli`. The binary is at `target/debug/cvkg`.
- **Port already in use**: The dev server defaults to port 3000. Use `--port 8080` to change.
- **File watcher errors**: Ensure `inotify` limits are sufficient on Linux: `echo 524288 | sudo tee /proc/sys/fs/inotify/max_user_watches`.
