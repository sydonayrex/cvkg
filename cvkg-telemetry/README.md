# cvkg-telemetry

## Purpose
Aggregates performance statistics, input latency tracks, and frame duration metrics.

## Boundaries
- It does not render dashboard layouts or draw visual diagrams.
- It does not contain testing frameworks; quality checks are managed by `cvkg-test`.

## Dependency Graph
```mermaid
graph TD
    cvkg-telemetry["cvkg-telemetry (Focal Crate)"]
    cvkg-core["cvkg-core"]
    cvkg-telemetry --> cvkg-core
    classDef focal fill:#0f172a,stroke:#3b82f6,color:#38bdf8,stroke-width:2px
    classDef sibling fill:#311042,stroke:#d946ef,color:#f472b6,stroke-width:1px
    class cvkg-telemetry focal
    class cvkg-core sibling
```

## Public API Overview
- `TelemetryClient` — Performance client tracker.
- `InputLatencyTracker` — Input percentile calculator.

## Usage Example
```rust
use cvkg_telemetry::TelemetryClient;
```

## Use Cases
- Mapped as a core component inside the standard framework dependency tree.

## Edge Cases and Limitations
- Under extreme scale or thread contention, ensure the host runtime balances cycles appropriately.

## Crate-Specific Build Flags
This crate has no custom feature flags or compile-time options. It compiles under standard cargo parameters.
