# ADR 0001: Composed element identity

Status: accepted

## Context

Focus, accessibility, and input dispatch currently derive identity from component-local hashes. The hashes cover partial inputs, so two widgets can collide. GPUI composes element IDs from ancestors and gets stable identity for the accessibility tree and focus routing.

## Decision

CVKG will use a composed element ID: a stack of `(parent, local_name)` pairs that is unique per frame. The ID feeds focus handles, accessibility node IDs, and action routing. Component-local hashes remain as fallback for legacy widgets until their migration lands.

## Consequences

A11y tree builders can express parent-child relations without duplicate labels. Focus order becomes explicit. Code that hashes only a placeholder string must migrate.

## Alternatives

Keep hash-based identity and add a global collision table. Rejected because collisions stay possible and the table duplicates the element tree.
