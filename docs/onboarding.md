# Onboarding Guide

Welcome to the Cyber Viking Kvasir Graph (CVKG) development environment. This guide will walk you through setting up your local machine and running your first "Berserker" application.

## 1. Prerequisites

### Rust Toolchain
CVKG requires **Rust 1.85+** (Stable or Nightly). We use the 2024 Edition.
```bash
rustup update
```

### System Dependencies (Linux)
If you are on Linux, you need the following libraries for windowing and GPU support:
```bash
sudo apt-get install -y libwayland-dev libx11-dev libxkbcommon-dev libasound2-dev libfontconfig1-dev
```

### GPU Support
Ensure your drivers are up to date. CVKG uses `wgpu` and requires a GPU supporting Vulkan, Metal, or DX12.

## 2. Workspace Setup

Clone the repository and verify the workspace:
```bash
git clone https://github.com/sydonayrex/cvkg.git
cd cvkg
cargo check
```

## 3. Running the Demos

The best way to see CVKG in action is to run the native demos:

### Berserker Fire Demo
A high-fidelity stress test of the GPU pipeline:
```bash
cargo run -p berserker
```

### Ulfhednar IDE Prototype
A demonstration of complex component composition:
```bash
cargo run -p ulfhednar
```

## 4. Development Workflow

### Creating a New Project
Use the CVKG CLI to scaffold a new application:
```bash
cargo run -p cvkg-cli -- new my-app
```

### Starting the Dev Server
For web development, start the WebKit server:
```bash
cargo run -p cvkg-cli -- dev --target wasm
```

## 5. Learning Path

1. **Architecture**: Read [architecture.md](./architecture.md) to understand the rendering pipeline.
2. **Components**: Explore [cvkg-components/README.md](../cvkg-components/README.md) for available UI elements.
3. **Macros**: Learn how to write declarative UI in [cvkg-macros/README.md](../cvkg-macros/README.md).
4. **Themes**: Customize your app's look in [cvkg-themes/README.md](../cvkg-themes/README.md).

## 6. Community & Support

- **Bugs**: Open an issue on GitHub.
- **Discussions**: Use the GitHub Discussions tab for architectural questions.
- **Contributions**: Follow the "CVKG Agentic Development Guidelines" (found in crate headers) for all pull requests.

Skål!