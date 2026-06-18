# P1-13: cvkg-core lib.rs Module Extraction Plan

## Current State
- **File:** cvkg-core/src/lib.rs (9,741 lines, 333KB)
- **Existing sub-modules:** error_types.rs, future_views.rs, security.rs
- **Inline modules:** color, audio_haptic, parallax

## Module Extraction Map

### Phase 1: Extract self-contained types (lowest risk)

Each of these has minimal dependencies on other sections of lib.rs:

#### 1a. `undo.rs` (lines ~439-640)
- UndoGroup, UndoManager
- Dependencies: none (standalone)

#### 1b. `window.rs` (lines ~643-780)
- WindowId, WindowLevel, WindowConfig, Window trait, WindowHandle, WindowCloseAction
- Dependencies: none

#### 1c. `asset.rs` (lines ~785-890)
- AssetKey, AssetState, TokenValue, YggdrasilTokens
- Dependencies: none

#### 1d. `error_boundary.rs` (lines ~47-258)
- ComponentErrorState, ErrorBoundary
- Dependencies: View trait (line 892)

#### 1e. `knowledge.rs` (lines ~262-470)
- KnowledgeState, KnowledgeId, KnowledgeFragment, MemoryLayer, Realm, AnnouncementPriority, TemporalNode, TemporalEdge
- Dependencies: none

### Phase 2: Extract core traits and view system (medium risk)

#### 2a. `view.rs` (lines ~892-1227)
- View trait, ViewModifier trait, MemoView, AnyView
- Dependencies: Rect, ColorTheme, etc. (need to extract first)

#### 2b. `renderer.rs` (lines ~2954-3400)
- Renderer trait (~50 methods)
- Dependencies: View trait, Rect, ColorTheme, Mesh, Material3D, etc.

#### 2c. `accessibility.rs` (lines ~1228-1362)
- AriaRole, AriaProperties
- Dependencies: none

#### 2d. `focus.rs` (lines ~1365-1549)
- KeyModifiers, KeyShortcut, FocusableId, FocusTrap, FocusManager
- Dependencies: none

### Phase 3: Extract event system (medium risk)

#### 3a. `event.rs` (lines ~5893-6144)
- Event enum, EventResponse, EventPhase, TouchPhase
- Dependencies: Rect

#### 3b. `asset_manager.rs` (lines ~5893-6234)
- AssetManager trait, DefaultAssetManager
- Dependencies: AssetKey, AssetState (from Phase 1c)

### Phase 4: Extract UI infrastructure (higher risk)

#### 4a. `suspense.rs` (lines ~6235-6925)
- Suspense<T>, BerserkerMode, Seer trait
- Dependencies: View trait

#### 4b. `clipboard.rs` (lines ~7459-7504)
- ClipboardProvider trait, SystemClipboard
- Dependencies: none

#### 4c. `text_input.rs` (lines ~7505-7698)
- TextDirection, TextInputState
- Dependencies: none

#### 4d. `notification.rs` (lines ~7699-7839)
- Notification, NotificationAction, NotificationPriority, NotificationError, NotificationPermission, NotificationHandler
- Dependencies: none

#### 4e. `file_dialog.rs` (lines ~7840-7977)
- FileFilter, FileDialogMode, FileDialog, FileDialogError, DocumentError, Document trait, AutoSaveManager
- Dependencies: none

#### 4f. `menu.rs` (lines ~8116-8369)
- MenuItem, MenuBar
- Dependencies: none

#### 4g. `l10n.rs` (lines ~8379-8556)
- L10nBundle, L10n, SystemTheme
- Dependencies: none

### Phase 5: Extract state management (higher risk)

#### 5a. `state.rs` (lines ~393-470 + 8066-8115)
- State<T>, SubscriberList, KnowledgeState
- Dependencies: KvasirId, DirtyFlags

#### 5b. `dirty.rs` (lines ~8680-8907)
- DirtyFlags, InvalidationRecord
- Dependencies: KvasirId

### Phase 6: Extract remaining utilities

#### 6a. `virtualization.rs` (lines ~9618-9741)
- VirtualizationConfig
- Dependencies: none

#### 6b. `frame_budget.rs` (lines ~9336-9566)
- FrameBudgetTracker, SubsystemBudget
- Dependencies: none

#### 6c. `dirty_region.rs` (lines ~9143-9335)
- DirtyRegionManager
- Dependencies: Rect

## Execution Order

The extraction must follow dependency order. Each phase can be
done in parallel within itself, but phases must be sequential:

**Phase 1** (types with no deps) -> commit
**Phase 2** (core traits) -> commit
**Phase 3** (events) -> commit
**Phase 4** (UI infrastructure) -> commit
**Phase 5** (state management) -> commit
**Phase 6** (utilities) -> commit

## Strategy

For each extraction:
1. Create the new module file in `cvkg-core/src/`
2. Move the code from lib.rs to the new file
3. Add `pub mod` and `pub use` declarations in lib.rs
4. Run `cargo check -p cvkg-core` to verify
5. Run `cargo test -p cvkg-core` to verify
6. Commit with message "refactor(P1-13): extract <module> from lib.rs"

## Risk Mitigation

- Keep all re-exports in lib.rs so downstream crates don't break
- Use `pub use` to maintain the existing API surface
- Each extraction is a single commit that can be reverted
- Test after every extraction
- The View and Renderer traits are the riskiest extractions
  since they're implemented by downstream crates

## Estimated Effort

- Phase 1: ~1 hour (5 small modules)
- Phase 2: ~2 hours (4 medium modules)
- Phase 3: ~1 hour (2 modules)
- Phase 4: ~2 hours (7 modules)
- Phase 5: ~1 hour (2 modules)
- Phase 6: ~30 min (3 modules)

Total: ~7-8 hours of focused work
