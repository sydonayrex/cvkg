# Norse Names Without English Aliases

The following Norse-named types do NOT have English aliases in the current
codebase. They are not used as type identifiers in examples (only in string
literals or comments), so no example changes were needed.

## Component types without aliases

These types are exported from `cvkg-components` but have no `pub type EnglishName = NorseName` alias:

- `LokiGlitch` -- Visual effect (glitch distortion)
- `NiflheimFrost` -- Visual effect (frost/glass)
- `Seiðr` -- Visual effect (holographic scanlines)
- `TyrSecurity` -- Security/permission component
- `HlinAccessibility` -- Accessibility infrastructure
- `A11yBeacon` -- Accessibility beacon
- `A11yInspector` -- Accessibility inspector
- `AwaitVeil` -- Loading placeholder
- `ComputedSignal` -- Reactive derived state
- `ConsentGate` -- GDPR/CCPA consent
- `DropVault` -- File drop zone
- `Editable` -- Inline text editing
- `FormBinder` -- Form state binding
- `HoverCard` -- Hover card overlay
- `InputGroup` -- Input group wrapper
- `InputOTP` -- One-time password input
- `MentionInput` -- @mention input
- `NativeSelect` -- Native select dropdown
- `PhoneInput` -- Phone number input
- `Popconfirm` -- Confirmation popover
- `QRCode` -- QR code generator
- `Sonner` -- Toast notification system
- `ToggleGroup` -- Toggle button group
- `FontAxisPanel` -- Variable font axis inspector
- `MjolnirFrame` -- Geometric frame effect
- `MjolnirSlider` -- Slider with geometric effects
- `MultiAgentOrchestrator` -- Multi-agent coordination
- `RadialMenu` -- Radial context menu
- `RichTreeView` -- Rich tree view
- `RunestoneEditor` -- Text editor component
- `SemanticMemoryExplorer` -- Memory exploration UI
- `ShieldWall` -- Defensive layout component
- `TextEditor` -- Full text editor
- `TimelineEditor` -- Timeline editing
- `Trustmark` -- Trust/verification badge
- `GeriTransfer` -- File transfer component
- `HringrPagination` -- Pagination (alias exists: `Pagination`)
- `ScribingStone` -- Note-taking component (alias exists: `ScribingNote`)
- `DraumaSkeleton` -- Skeleton loading placeholder
- `MerkiBadge` -- Badge component
- `Vegvísir` -- Navigation compass component
- `HeimdallSweep` -- Radar sweep effect
- `FutharkFlow` -- Flowing rune effect
- `RunesCard` -- Card component
- `EikonaForm` -- Form component

## Notes

- `Bifrost` appears in string literals ("Enable Bifrost", "Project Bifrost Security Protocol") but these are not type identifiers
- `Mjolnir` appears in comments and string literals but not as a type identifier in examples
- `Saga` appears in `SagaAccordion` which has an alias `Accordion` -- already replaced in examples
