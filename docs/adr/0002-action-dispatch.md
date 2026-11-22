# ADR 0002: Central action dispatch

Status: accepted

## Context

`View::on_key_event` exists on four component types but has no production caller. `KeyShortcut` can parse shortcuts, but no code maps them to typed actions. Web and native input paths implement their own shortcut tables.

## Decision

One dispatcher in `cvkg-core` owns a key context stack and typed action handlers. Native and web input feed the same dispatcher. The dispatcher resolves a shortcut, invokes the matching action handler, then calls `on_key_event` on the focused view.

## Consequences

Keyboard behavior becomes testable through public APIs. Component handlers gain a production caller. Existing per-platform shortcut tables stay as fallback for one release.

## Alternatives

Add key handling to each component and each platform separately. Rejected because behavior drifts and the existing `on_key_event` contract stays dead.
