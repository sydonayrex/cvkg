# cvkg-layout TLDR.md

## Purpose
Own the layout engine: FlexBox, Grid, Container queries (FlexiScope), padding, spacing, and the layout computation system built on Taffy.

## Ownership
- `src/lib.rs` — Layout primitives, Taffy integration, SizeProposal/Rect types
- FlexBox, Grid, Padding, SafeArea, AspectRatio, Spacer
- FlexiScope container queries

## Local Contracts
- Layout computation must be deterministic and idempotent.
- Container queries must read container width, not viewport width.
- LayoutCache must be invalidated correctly on constraint changes.

## Verification
- Run `cargo test -p cvkg-layout`
- Run `cargo check --workspace`
