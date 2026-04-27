# CVKG Security Threat Model (v1.0)

## 1. Overview
This document outlines the security architecture, threat model, and mitigation strategies for the CVKG ecosystem, with a focus on plugin isolation and capability-based access control.

## 2. Trust Boundaries
- **Core (Trusted)**: The CVKG core runtime, layout engine, and official renderers are considered trusted.
- **First-Party Plugins (Trusted)**: Official components provided by the CVKG team are considered trusted but still subject to standard capability auditing.
- **Third-Party Plugins (Untrusted)**: All components or modifiers provided by external authors or loaded at runtime MUST be treated as untrusted.

## 3. Threat Scenarios

### T1: Arbitrary Code Execution (Sandbox Escape)
- **Description**: An untrusted plugin attempts to execute code outside the WASM sandbox or bypass host function restrictions.
- **Mitigation**: Plugins are executed in a strictly metered WASM runtime (e.g., wasmtime). Direct memory access to the host is blocked. All communication occurs via a serialized message interface.

### T2: Unauthorized Resource Access (Capability Violation)
- **Description**: A plugin attempts to access the filesystem, network, or agentic reasoning without explicit permission in its manifest.
- **Mitigation**: Capability-based security model. Plugins MUST declare required capabilities in `PluginManifest`. The host runtime enforces these checks at the entry point of every sensitive operation.

### T3: Denial of Service (Resource Exhaustion)
- **Description**: A plugin consumes excessive CPU or memory, hanging the UI thread or crashing the process.
- **Mitigation**: Strict `SandboxLimits` (max memory, max CPU time per frame). The runtime terminates plugins that exceed their quotas.

### T4: Sensitive Data Leakage
- **Description**: A plugin reads global system state or user secrets and exfiltrates them via the network.
- **Mitigation**: State access is filtered based on the plugin's context. Network access is restricted to authenticated/encrypted endpoints declared in the manifest.

## 4. Security Guarantees
- **Isolation**: Plugin crashes or panics do NOT crash the host application.
- **Privacy**: Plugins only see the portion of the state graph they are explicitly allowed to observe.
- **Auditability**: All capability requests and enforcement actions are logged for developer review.

## 5. Developer Requirements
- Plugin authors MUST use the official `cvkg-plugin-sdk`.
- All network communication MUST use HTTPS/WSS.
- Sensitive data MUST never be stored in plaintext in the plugin's local state.
