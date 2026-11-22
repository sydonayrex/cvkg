# ADR 0003: Single translation owner

Status: accepted

## Context

`GpuRenderer` and `VNodeRenderer` both wrote the thread-local renderer translation. Primitives read the global while `current_translation()` read a local stack, so the two views of current translation could disagree.

## Decision

`GpuRenderer` owns cumulative translation. `VNodeRenderer` keeps capture-time geometry but does not write the renderer thread-local. Frame reset clears the stack and the global. All primitives read one source.

## Consequences

Nested local-coordinate layouts produce one cumulative offset. An unbalanced push cannot bias the next frame. `push_translation` stays an opt-in API until an action-based caller exists.

## Alternatives

Keep two stacks and synchronize them on every push. Rejected because last-writer-wins behavior remains ambiguous during concurrent capture and paint passes.
