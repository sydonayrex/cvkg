# CVKG AGENTS.md

## Purpose
Root DOX rail for the CVKG Rust workspace. Keep rendering, UI, GPU, and audit artifacts understandable from this file plus the nearest child AGENTS.md.

## Ownership
- Root workspace: owned by the CVKG maintainers through this file.
- Rendering/GPU: `cvkg-render-gpu`, `cvkg-render-native`, `cvkg-compositor`, `cvkg-test`, and demo crates.
- UI components/layout: `cvkg-components`, `cvkg-layout`, `cvkg-vdom`, `cvkg-themes`, `cvkg-runic-text`.
- Cross-cutting core: `cvkg-core`, `cvkg-scene`, `cvkg-anim`, `cvkg-flow`, `cvkg-svg-*`, `cvkg-cli`.

## Local Contracts
- Read the root DOX rail before editing. Then read the nearest applicable child AGENTS.md and all parent docs on the path to the target.
- For GPU or render graph work, read `cvkg-render-gpu/AGENTS.md`.
- Do not use subagents for this repository unless the user explicitly allows it.
- Prefer completing or wiring features over deleting them. Stubs, TODOs, and unfinished logic must be reported and fixed where possible.
- Keep user-facing terminal output concise, ASCII, and production-grade.

## Work Guidance
- For audits, verify actual code paths with direct file reads and cargo metadata. Do not rely on stale summaries or git index state when the working tree is dirty.
- For rendering/UI audits, inspect Rust pipeline code, WGSL shaders, tests, Cargo manifests, and dependency graphs.
- For Tahoe-level UI readiness, evaluate rendering quality, accessibility, animation, component completeness, performance, and maintainability.

## Verification
- Run relevant cargo checks/tests for touched crates.
- For rendering work, verify both compile-time and render output when possible.
- For docs-only changes, verify paths and graph/report contents exist.

## Child DOX Index
- `cvkg-render-gpu/AGENTS.md` - GPU render graph, Surtr renderer, WGSL shader, resource lifecycle, Tahoe glass/backdrop pipeline.
- `cvkg-components/src/agent_chat.rs` - Agent/AI chat components: AgentChat, MessageList, InputBar, UserMessage, AssistantMessage, Markdown, ToolCard, SuggestionChips, ModelPicker, CopyToolbar, TextShimmer.
- `cvkg-components/src/text_anim.rs` - Text animations and card/button effects: TextAnimate, TypewriterEffect, NumberTicker, CardStack, CardHoverEffect, ExpandableCard, DraggableCard, ShimmerButton, RippleButton, StatefulButton.
- `cvkg-components/src/layout_components.rs` - Layout, navigation, carousel: BentoGrid, FloatingNavbar, NavbarMenu, Loader, MultiStepLoader, Carousel, Marquee.
- `cvkg-components/src/m3_components.rs` - Material 3, Cult UI, Joy UI, data: FAB, ExtendedFAB, TimePicker, DateRangePicker, HeroColorPanels, BgMediaHero, LogoCarousel, DynamicIsland, SidePanel, Codeblock, Kanban.
