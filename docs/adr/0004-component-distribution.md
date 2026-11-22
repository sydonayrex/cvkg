# ADR 0004: Crates canonical, source distribution additive

Status: accepted

## Context

Rust/UI distributes copy-paste component source. CVKG publishes crates. Users cannot inspect or diff a single component's source today.

## Decision

Published crates stay canonical. A generated manifest and `cvkg-cli component` commands let users view, copy, and diff one component source. The CLI reads the manifest generated from `cvkg-components/src`.

## Consequences

Users get registry ergonomics without losing type-checked crate APIs. CI must verify that manifest hashes match source files.

## Alternatives

Replace crate distribution with copy-paste source. Rejected because CVKG components depend on shared traits and tokens that copy-paste alone cannot guarantee.
