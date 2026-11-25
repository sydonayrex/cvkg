# ADR 0005: Entity ownership

- Status: Accepted, gallery frame path migrated; write path still uses `State`/mutex
- Date: 2026-11-22

## Context

`State<T>` gives cvkg reactive values with cheap clones. It does not give an
entity identity that survives value replacement, nor an explicit point where
observers run. The cross-review found gallery state behind one global
`Arc<Mutex<GalleryState>>` with locks spread across the frame path.

## Decision

`cvkg-core` owns `Entity<T>` and `EntityArena<T>`. A handle carries an index and
a generation, so a removed entity cannot alias a later value at the same slot.
Mutation and notification are separate: `update` changes the stored value,
`notify` runs each observer once with a snapshot of that value.

`State<T>` stays unchanged. It remains the right primitive for lightweight
reactive values. Entity handles are for app-owned state whose identity matters.

## Consequences

- Observers see one notification per explicit `notify` call. Batched updates
  can therefore collapse multiple mutations into one observation.
- A stale handle fails reads and notifications instead of touching a recycled
  slot.
- Callbacks are `Send + Sync` and stored as `Arc`, so the arena is safe to share
  between threads. Callers must not mutate the arena from inside a callback.

## Remaining work

`GalleryApp::render` publishes one entity-backed snapshot per frame, and
`GalleryCanvas` reads that handle without taking the global mutex. Writes still
go through `Arc<Mutex<GalleryState>>`. Migrating the write path to entity
`update` calls is the next step; keep `State<T>` as the compatibility path
until both the gallery and demos use entity handles in production.
