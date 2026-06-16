# cvkg-skills AGENTS.md

## Purpose
Own the skills system: skill definitions, skill loading, and the skill execution framework for AI-driven development workflows.

## Ownership
- `src/lib.rs` — Skill trait, skill registry, skill execution
- Integration with the Hermes agent system

## Local Contracts
- Skills must be loadable at runtime without recompilation.
- Skill execution must be sandboxed and capability-restricted.
- Skill metadata must be self-describing for agent consumption.

## Verification
- Run `cargo test -p cvkg-skills`
- Run `cargo check --workspace`
