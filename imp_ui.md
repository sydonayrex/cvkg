# CVKG Implementation Plan — Items 1 through 17

**Last Updated**: February 2026
**Goal**: Make CVKG a futuristic, modern, and compelling UI/UX framework
**Baseline**: 20,600 lines across 12 crates, 0 build errors, Phases 1-4 complete
**Status Key**: ✅ Complete | 🔄 In Progress | ⬜ Not Started

---

## Priority Ordering

Items are ordered by dependency graph and impact. Text input (#1) is foundational — every app needs it. Layout (#2) is next because nothing can be positioned correctly without it. Then the platform integrations that make it feel native.

---

## Item 1: Text Input System (CRITICAL) ✅

**Status**: ✅ Complete
**Date completed**: February 2026

**What was done**:

### 1.1 — TextInput types in cvkg-core ✅

Added to `cvkg-core/src/lib.rs`:
- `ClipboardProvider` trait with `SystemClipboard` implementation (uses macOS `pbcopy`/`pbpaste`)
- `TextDirection` enum (Forward, Backward, Up, Down, LineStart, LineEnd, WordForward, WordBackward)
- `TextInputState` struct with full text editing operations:
  - `new(text)` — create with initial text
  - `insert(text)` — insert at cursor, replacing selection
  - `delete(backward, count)` — delete characters or selection
  - `move_cursor(direction, extend)` — move cursor with optional selection extension
  - `select_all()` — select entire text
  - `selection_range()` → `Option<(usize, usize)>` — get selection bounds
  - `selected_text()` → `String` — get selected text content
  - `cursor_byte_pos()` → `usize` — get cursor position
- All UTF-8 safe character boundary handling
- Word navigation (skips word characters then whitespace)

### 1.2 — Input component rewrite ✅

Replaced the Input component in `cvkg-components/src/interactive.rs`:
- New struct with `on_commit` callback, `state_id` for system state tracking
- Full cursor rendering with blinking (1Hz sine wave alpha)
- Selection rendering with accent-colored highlight `[accent[0], accent[1], accent[2], 0.3]`
- Key handling: character input, backspace, delete, arrow keys, home/end, enter
- IME composition handler
- Pointer click to position cursor
- Theme-aware colors for all states (uses `theme::accent()`, `theme::text()`, etc.)
- Each handler wrapped in its own `{ let on_change = self.on_change.clone(); ... }` block to avoid Arc move issues

### 1.3 — InputState enum fix ✅

Added `Clone` derive to `InputState` enum (was needed by the new Input struct).

### 1.4 — Verification

- `cargo check -p cvkg-core` — 0 errors
- `cargo check -p cvkg-components` — 0 errors
- `cargo check --workspace` — 0 errors

**Files modified**:
- `cvkg-core/src/lib.rs` — +230 lines (ClipboardProvider, TextDirection, TextInputState)
- `cvkg-components/src/interactive.rs` — Input component rewritten (~400 lines replaced)

---

## Item 2: Layout Engine Completion (CRITICAL)

**Status**: ✅ Complete — Grid (with tests), ScrollView (rubber-band physics, momentum, scrollbars), OverlayModifier, PaddingModifier, FrameModifier all implemented in cvkg-layout and cvkg-core

**Current state**: HStack, VStack, ZStack, Flex exist with basic flex weight distribution. Missing: padding, spacing constraints, alignment overlays, intrinsic size propagation, and scroll layout.

### 2.1 — Padding modifier (cvkg-core)
```rust
/// Insert text at the current cursor position (or replace selection)
fn text_input_insert(&mut self, text: &str);
/// Delete characters backward (backspace) or forward (delete)
fn text_input_delete(&mut self, backward: bool, count: usize) -> String;
/// Move the cursor. `extend` extends the selection anchor.
fn text_input_move_cursor(&mut self, direction: TextDirection, extend: bool);
/// Select all text
fn text_input_select_all(&mut self);
/// Get the current cursor position (byte offset into the text)
fn text_input_cursor_pos(&self) -> usize;
/// Get the selection range (start, end) — None if no selection
fn text_input_selection(&self) -> Option<(usize, usize)>;
/// Get the currently selected text
fn text_input_selected_text(&self) -> String;
/// Copy selection to clipboard
fn text_input_copy(&self) -> String;
/// Paste from clipboard at cursor, replacing selection
fn text_input_paste(&mut self, text: &str);
/// Cut: copy + delete selection
fn text_input_cut(&mut self) -> String;

pub enum TextDirection {
    Forward,
    Backward,
    Up,
    Down,
    LineStart,
    LineEnd,
    WordForward,
    WordBackward,
}
```

### 1.2 — Input component rewrite (cvkg-components/src/interactive.rs)

The current Input render method is ~100 lines with basic cursor tracking. It needs to become ~400 lines with:

- **Text buffer management**: Store `String` + `cursor_pos: usize` + `selection_anchor: Option<usize>` in component state via system state hash
- **Caret rendering**: Blinking block cursor (1Hz, toggled by focus state). Draw as `fill_rect` at the cursor position computed via `measure_text` on the substring before cursor
- **Selection rendering**: Highlight selected text range with `fill_rect` behind the text, using accent color at 0.3 alpha
- **Key handling**:
  - Arrows: move cursor by character (or word with Ctrl/Cmd)
  - Home/End: move to line start/end
  - Backspace/Delete: remove character before/after cursor
  - Ctrl+A/Cmd+A: select all
  - Ctrl+C/Cmd+C: copy to clipboard
  - Ctrl+V/Cmd+V: paste from clipboard
  - Ctrl+X/Cmd+X: cut to clipboard
  - Enter: fire `on_commit` callback (for single-line) or insert newline (for multi-line)
- **Pointer interaction**: Click to position cursor, drag to select, double-click to select word
- **Undo coalescing**: Characters typed within 500ms coalesce into one undo group via `cvkg-anim::SleipnirSolver` integration
- **IME composition**: Show preedit string with underline, commit on Enter

State structure:
```rust
struct InputState {
    text: String,
    cursor_pos: usize,
    selection_anchor: Option<usize>,  // None = no selection
    focused: bool,
    caret_visible: bool,              // For blinking
    last_edit_time: f32,              // For undo coalescing
    undo_stack: Vec<UndoEntry>,
}

struct UndoEntry {
    text: String,
    cursor_pos: usize,
    timestamp: f32,
}
```

### 1.3 — Textarea component (NEW, cvkg-components/src/interactive.rs)

Multi-line variant of Input:
- Word wrapping via `cvkg-runic-text` line breaking
- Vertical scrolling within fixed height
- Tab key inserts spaces instead of changing focus
- Same selection/copy/paste/undo as Input
- Uses `text_input_move_cursor(Up/Down)` for vertical navigation

### 1.4 — Clipboard trait (cvkg-core)

```rust
pub trait ClipboardProvider: Send + Sync {
    fn read_text(&self) -> Option<String>;
    fn write_text(&self, text: &str);
}

// Default implementation via arboard
pub struct SystemClipboard;

// Stored in Renderer as Option<Arc<dyn ClipboardProvider>>
```

### 1.5 — Renderer implementation (cvkg-render-gpu)

Implement the `TextInput` trait methods on `NativeRenderer`:
- Track `TextInputState` with text buffer, cursor, selection
- On `text_input_insert`, mutate the buffer and re-measure
- On `copy`, read `arboard::Clipboard` and return selected text
- On `paste`, read clipboard and insert at cursor

**Verification**: `cargo check --workspace` passes. A demo app can type text, select with mouse drag, copy/paste, and use arrow keys.

---

## Item 2: Layout Engine Completion (CRITICAL)

**Current state**: HStack, VStack, ZStack, Flex exist with basic flex weight distribution. Missing: padding, spacing constraints, alignment overlays, intrinsic size propagation, and scroll layout.

### 2.1 — Padding modifier (cvkg-core)

Already exists as `PaddingModifier`. Verify it correctly shrinks the proposal and transforms the rect. If not, fix:
```rust
fn transform_rect(&self, rect: Rect) -> Rect {
    Rect {
        x: rect.x + self.amount,
        y: rect.y + self.amount,
        width: (rect.width - 2.0 * self.amount).max(0.0),
        height: (rect.height - 2.0 * self.amount).max(0.0),
    }
}
```

### 2.2 — Frame modifier with alignment (cvkg-core)

Already exists as `FrameModifier`. Add alignment support:
```rust
pub struct FrameModifier {
    pub width: Option<f32>,
    pub height: Option<f32>,
    pub min_width: Option<f32>,
    pub max_width: Option<f32>,
    pub min_height: Option<f32>,
    pub max_height: Option<f32>,
    pub alignment: Alignment,  // New: how to align content within the frame
}
```

### 2.3 — ScrollView implementation (cvkit-components/src/container.rs)

Current `ScrollView` is a stub. Implement:
- **Content offset tracking**: `scroll_offset: [f32; 2]` in component state
- **Scroll gesture handling**: Track pointer drag distance, apply to scroll_offset with rubber-band physics
- **Rubber-band physics**: When scrolling past content bounds, apply resistance: `offset = bounds + (offset - bounds) * 0.3` with `cvkg-anim::SleipnirSolver`
- **Scroll indicators**: Render thin rectangles on right/bottom edges showing scroll position relative to content
- **Momentum scrolling**: On pointer release with velocity, animate offset with deceleration curve
- **Keyboard scrolling**: Page Up/Down, Home/End keys
- **Scrollbar interaction**: Click/drag on scrollbar track

```rust
pub struct ScrollView {
    content: Box<dyn View>,
    scroll_offset: [f32; 2],
    content_size: [f32; 2],
    viewport_size: [f32; 2],
    momentum_velocity: [f32; 2],
    scrollbar_width: f32,
    rubber_band_factor: f32,
}
```

### 2.4 — Grid layout (cvkg-layout/src/lib.rs)

New `Grid` type:
```rust
pub struct Grid {
    columns: Vec<GridTrack>,   // Track sizing: Fixed(f32), Flex(f32), Auto
    rows: Vec<GridTrack>,
    column_gap: f32,
    row_gap: f32,
}

pub enum GridTrack {
    Fixed(f32),    // Exact pixel width
    Flex(f32),     // Proportional to available space
    Auto,          // Size to fit content
    MinMax(f32, f32), // Clamp between min and max
}

// Child placement via modifier:
#[derive(Clone, Copy)]
pub struct GridPlacement {
    pub column: i32,       // 0-based, negative = from end
    pub column_span: u32,
    pub row: i32,
    pub row_span: u32,
}
```

### 2.5 — Overlay modifier (cvkg-core)

For tooltips, popovers, menus that render above other content:
```rust
pub struct OverlayModifier {
    pub alignment: Alignment,    // Where relative to the anchored view
    pub offset: [f32; 2],       // Additional offset
    pub on_dismiss: Option<Arc<dyn Fn() + Send + Sync>>, // Click-outside dismissal
}
```

**Verification**: `cargo check --workspace` passes. A demo shows a scrollable list, a grid of cards, and a popover.

---

## Item 3: Working Demo App (CRITICAL)

**Status**: ✅ Complete — demos/showcase/ created with 8 section pages, sidebar navigation, live theme switcher, accessibility preference toggles, OS-agnostic keyboard shortcuts (Cmd+T theme, Cmd+M motion, etc.) There's no way for a developer to see what CVKG can do.

### 3.1 — Create showcase demo (demos/showcase/)

Structure:
```
demos/showcase/
  Cargo.toml
  src/
    main.rs          # Entry point
    app.rs           # App shell with sidebar navigation
    pages/
      buttons.rs     # All button variants, states
      inputs.rs      # Text input, textarea, picker, stepper
      layout.rs      # HStack, VStack, Grid, ScrollView demos
      overlays.rs    # Tooltip, popover, dialog, sheet demo
      visual.rs      # Progress, gauge, skeleton, toast demo
      a11y.rs        # Accessibility preferences live toggle
      glass.rs       # Glass effects demo with wallpaper
```

### 3.2 — Sidebar navigation

Left sidebar (200px) with sections:
- Buttons, Inputs, Forms, Layout, Overlays, Visual, Accessibility, Glass

### 3.3 — Live theme switcher

Header bar with:
- Dark/Light mode toggle (calls `set_current_theme`)
- Reduce Motion toggle
- Reduce Transparency toggle
- Increase Contrast toggle
- Accent color picker

Each toggle immediately updates the UI to show the effect.

### 3.4 — Component pages

Each page shows:
- Component rendering with all variants
- Interactive controls to change state
- Source code snippet (read from file at build time via `include_str!`)
- Description of AccessKit role

### 3.5 — Glass demo page

Special page that:
- Renders a background image (wallpaper)
- Places glass panels over it
- Shows blur/refraction/specular effects
- Has sliders for blur radius, refraction strength, tint opacity

**Verification**: `cargo run -p showcase` launches a working app with all components visible and interactive.

---

## Item 4: Unified Undo/Redo System

**Status**: ✅ Complete — UndoManager + UndoGroup in cvkg-core with push/undo/redo/coalesce, depth limiting. Ad-hoc undo in Input also present. Input has ad-hoc undo stack. No global undo manager.

### 4.1 — UndoManager (cvkg-core)

```rust
pub struct UndoManager {
    stack: Vec<UndoGroup>,
    position: usize,          // Current position in stack
    max_depth: usize,         // Max undo steps (default: 100)
    coalesce_window: f32,     // Seconds to coalesce (default: 0.5)
}

pub struct UndoGroup {
    label: String,            // "Type", "Delete", "Paste", etc.
    timestamp: f32,
    undo: Arc<dyn Fn() + Send + Sync>,
    redo: Arc<dyn Fn() + Send + Sync>,
}

impl UndoManager {
    pub fn push(&mut self, label: &str, undo: impl Fn() + Send + Sync + 'static, redo: impl Fn() + Send + Sync + 'static);
    pub fn undo(&mut self) -> bool;
    pub fn redo(&mut self) -> bool;
    pub fn can_undo(&self) -> bool;
    pub fn can_redo(&self) -> bool;
    pub fn clear(&mut self);
    /// Merge with previous group if within coalesce window and same label
    pub fn push_coalesceable(&mut self, label: &str, undo: ..., redo: ...);
}
```

### 4.2 — Global undo manager in system state

Add to `KnowledgeState`:
```rust
pub undo_manager: UndoManager,
```

### 4.3 — Keyboard shortcuts for undo/redo

In native renderer, handle:
- Ctrl+Z / Cmd+Z → undo
- Ctrl+Shift+Z / Cmd+Shift+Z → redo
- Ctrl+Y / Cmd+Y → redo (Windows convention)

### 4.4 — Input integration

Replace Input's ad-hoc undo with `UndoManager::push_coalesceable("Type", undo, redo)`.

**Verification**: Text input supports multi-level undo/redo with coalescing.

---

## Item 5: Real Text Input with Selection (DEPENDS ON #1)

**Status**: ✅ Complete — Covered by Item 1. Input component has cursor, selection, clipboard, IME. OS-agnostic Cmd/Ctrl shortcuts wired in render-native.

This is the implementation of #1's design. Since #1 covers the architecture, this item is the actual implementation work.

### 5.1 — Implement TextInput trait on NativeRenderer (500 lines)

### 5.2 — Rewrite Input component (400 lines)

### 5.3 — Write Textarea component (300 lines)

### 5.4 — Integration tests

- Type "Hello World" → cursor at end
- Press Left 5 times → cursor moves
- Press Shift+Right → selection grows
- Press Backspace → deletes selection
- Press Ctrl+Z → undoes
- Paste from clipboard → inserts at cursor

**Estimated total**: ~1,200 lines of new code

---

## Item 6: Multi-Window Support

**Status**: ✅ Complete — WindowHandle (cvkg-core), WindowManager (cvkg-render-native) with per-window VDom, z-ordering, event dispatch. Cmd+N/Ctrl+N shortcut. but the API doesn't model windows as first-class citizens. There's no way to create, close, or manage windows from app code.

### 6.1 — Window API (cvkg-core)

```rust
pub struct WindowHandle {
    id: WindowId,
    inner: Arc<Window>,
}

pub struct WindowConfig {
    pub title: String,
    pub size: (f32, f32),
    pub min_size: Option<(f32, f32)>,
    pub max_size: Option<(f32, f32)>,
    pub resizable: bool,
    pub transparent: bool,
    pub decorations: bool,
    pub level: WindowLevel,       // Normal, AlwaysOnTop, PopUpMenu
}

pub enum WindowLevel {
    Normal,
    AlwaysOnTop,
    PopUpMenu,
}

impl WindowHandle {
    pub fn close(self);
    pub fn set_title(&self, title: &str);
    pub fn set_size(&self, width: f32, height: f32);
    pub fn is_key(&self) -> bool;
    pub fn is_main(&self) -> bool;
    pub fn is_visible(&self) -> bool;
    pub fn set_visible(&self, visible: bool);
    pub fn bring_to_front(&self);
}
```

### 6.2 — Window manager (cvkg-render-native)

```rust
pub struct WindowManager {
    windows: HashMap<WindowId, WindowData>,
    window_stack: Vec<WindowId>,     // Z-order (front to back)
}

impl WindowManager {
    pub fn create_window(&mut self, config: WindowConfig) -> WindowHandle;
    pub fn close_window(&mut self, id: WindowId);
    pub fn window(&self, id: WindowId) -> Option<&WindowData>;
    pub fn window_mut(&mut self, id: WindowId) -> Option<&mut WindowData>;
    pub fn window_order(&self) -> &[WindowId];
    pub fn bring_to_front(&mut self, id: WindowId);
}
```

### 6.3 — Window-level rendering

Each window gets its own:
- VDom instance
- Render pass (scene capture → blur → glass → overlay)
- Damage tracking
- Event dispatch

The renderer's `end_frame` iterates windows in z-order, rendering each to its own surface.

### 6.4 — Window close flow

1. User clicks close button
2. Window receives `CloseRequested` event
3. If `on_close` callback returns `WindowCloseAction::Confirm`, show confirmation dialog
4. If `Allow`, remove window from manager
5. If no windows remain, exit event loop (unless `quit_on_last_window_close` is false)

```rust
pub enum WindowCloseAction {
    Allow,      // Close immediately
    Confirm,    // Show confirmation dialog
    Deny,       // Don't close
}
```

### 6.5 — Keyboard shortcut: Cmd+N (new window)

In native renderer, handle `Cmd+N` to create a new window with default config.

**Verification**: `cargo run -p showcase` allows creating new windows via button, each with independent content.

---

## Item 7: Notification System

**Status**: ✅ Complete — Notification types (cvkg-core), Toast component (818 lines), NotificationCenterPanel (312 lines), ToastKind variants. OS-agnostic shortcuts. No integration with OS notification centers.

### 7.1 — Notification API (cvkg-core)

```rust
pub struct Notification {
    pub id: String,
    pub title: String,
    pub body: String,
    pub icon: Option<String>,       // Asset path
    pub sound: Option<String>,      // Sound name or path
    pub actions: Vec<NotificationAction>,
    pub timeout: Option<f32>,       // Auto-dismiss after N seconds
    pub priority: NotificationPriority,
}

pub struct NotificationAction {
    pub id: String,
    pub title: String,
    pub is_destructive: bool,
}

pub enum NotificationPriority {
    Passive,     // No alert, just in notification center
    Active,      // Alert but doesn't interrupt
    TimeSensitive, // Breaks through Focus modes
}

pub trait NotificationHandler: Send + Sync {
    pub fn show(&self, notification: Notification) -> Result<(), NotificationError>;
    pub fn dismiss(&self, id: &str);
    pub fn request_permission(&self) -> NotificationPermission;
}

pub enum NotificationPermission {
    Granted,
    Denied,
    NotDetermined,
}
```

### 7.2 — macOS notification backend (cvkg-render-native)

Use `objc2` crate to call `UNUserNotificationCenter`:
```rust
#[cfg(target_os = "macos")]
pub struct MacNotificationHandler;

#[cfg(target_os = "macos")]
impl NotificationHandler for MacNotificationHandler {
    fn show(&self, notification: Notification) -> Result<(), NotificationError> {
        // Create UNMutableNotificationContent
        // Set title, body, sound
        // Create UNTimeIntervalNotificationTrigger if timeout
        // Add to UNUserNotificationCenter
    }
}
```

### 7.3 — Toast component (cvkg-components/src/toast.rs)

In-app toast that mirrors the OS notification:
```rust
pub struct ToastManager {
    toasts: Vec<ToastEntry>,
    max_visible: usize,
    position: ToastPosition,
}

pub struct ToastEntry {
    notification: Notification,
    state: ToastState,
    progress: f32,      // 0.0 to 1.0 for auto-dismiss timer
}

pub enum ToastState {
    Entering,   // Slide in animation
    Visible,
    Exiting,    // Slide out animation
}

pub enum ToastPosition {
    TopLeading,
    TopCenter,
    TopTrailing,
    BottomLeading,
    BottomCenter,
    BottomTrailing,
}
```

### 7.4 — Notification center panel

A slide-out panel (like macOS Notification Center) showing:
- Recent notifications grouped by app
- Dismiss buttons
- Action buttons
- "Clear All" button

**Verification**: Show a button that triggers a notification. It appears as both an OS notification and an in-app toast.

---

## Item 8: File Operations

**Status**: ✅ Complete — FileDialog with OpenFile/OpenDirectory/SaveFile modes, filters, FileDialogError. Cmd+O/Ctrl+O and Cmd+S/Ctrl+S wired. No drag-and-drop file handling.

### 8.1 — FileDialog API (cvkg-core)

```rust
pub struct FileDialog {
    pub title: String,
    pub default_path: Option<String>,
    pub filters: Vec<FileFilter>,
    pub mode: FileDialogMode,
    pub allow_multiple: bool,
}

pub struct FileFilter {
    pub name: String,
    pub extensions: Vec<String>,  // e.g., ["png", "jpg", "jpeg"]
}

pub enum FileDialogMode {
    OpenFile,
    OpenDirectory,
    SaveFile,
}

impl FileDialog {
    pub fn new(mode: FileDialogMode) -> Self;
    pub fn title(mut self, title: impl Into<String>) -> Self;
    pub fn add_filter(mut self, name: &str, extensions: &[&str]) -> Self;
    pub fn default_path(mut self, path: impl Into<String>) -> Self;
    pub fn pick(self) -> Result<Vec<PathBuf>, FileDialogError>;
    pub fn pick_single(self) -> Result<Option<PathBuf>, FileDialogError>;
}

pub enum FileDialogError {
    Cancelled,
    Io(std::io::Error),
    Platform(String),
}
```

### 8.2 — Native implementation (cvkg-render-native)

Use `rfd` crate (already a workspace dependency):
```rust
#[cfg(not(target_arch = "wasm32"))]
impl FileDialog {
    pub fn pick(self) -> Result<Vec<PathBuf>, FileDialogError> {
        let mut dialog = rfd::FileDialog::new();
        dialog = dialog.set_title(&self.title);
        if let Some(path) = &self.default_path {
            dialog = dialog.set_directory(path);
        }
        for filter in &self.filters {
            dialog = dialog.add_filter(&filter.name, &filter.extensions);
        }
        match self.mode {
            FileDialogMode::OpenFile => {
                if self.allow_multiple {
                    dialog.pick_files().ok_or(FileDialogError::Cancelled)
                } else {
                    Ok(dialog.pick_file().into_iter().collect())
                }
            }
            // ... similar for OpenDirectory, SaveFile
        }
    }
}
```

### 8.3 — File drop handler (cvkg-render-native)

Handle `WindowEvent::DroppedFile`:
```rust
WindowEvent::DroppedFile(path) => {
    if let Some(vdom) = &state.vdom {
        vdom.dispatch_event(cvkg_core::Event::FileDrop {
            path: path.to_string_lossy().into_owned(),
        });
    }
}
```

Add to Event enum:
```rust
FileDrop { path: String },
```

### 8.4 — Document-based app model (cvkg-core)

```rust
pub trait Document: Send + Sync {
    fn read_from(path: &Path) -> Result<Self, DocumentError> where Self: Sized;
    fn write_to(&self, path: &Path) -> Result<(), DocumentError>;
    fn is_dirty(&self) -> bool;
    fn mark_clean(&mut self);
}

pub enum DocumentError {
    Io(std::io::Error),
    Parse(String),
    Serialize(String),
}

// Auto-save timer
pub struct AutoSaveManager {
    interval: f32,  // seconds
    documents: Vec<Box<dyn Document>>,
}
```

### 8.5 — File manager component (cvkg-components/src/file_tree.rs)

Enhance existing `YggdrasilTree`:
- File type icons based on extension
- Context menu (right-click): Open, Rename, Delete, Copy Path
- Drag-and-drop reordering
- Inline rename on slow double-click (Tahoe spring-loaded folders)
- Selection with Shift+Click (range) and Cmd+Click (toggle)

**Verification**: A file manager demo shows the filesystem, allows opening files, and supports drag-and-drop.

---

## Item 9: Menu Bar Integration

**Status**: ✅ Complete — MenuBar, MenuItem (Action/Submenu/Separator), KeyboardShortcut with OS-agnostic cmd modifier (Cmd on macOS, Ctrl on Windows/Linux), Modifiers struct, standard() constructor with all conventional shortcuts. Native menu rendering in cvkg-render-native via build_native_menu().

**Current state**: No native menu bar. The `cvkg-components/src/wyrd_hud.rs` has a `MuninMenubar` but it's not connected to the OS menu bar.

### 9.1 — Menu bar API (cvkg-core)

```rust
pub struct MenuBar {
    items: Vec<MenuItem>,
}

pub enum MenuItem {
    Action {
        label: String,
        shortcut: Option<KeyboardShortcut>,
        action: Arc<dyn Fn() + Send + Sync>,
        enabled: bool,
    },
    Submenu {
        label: String,
        items: Vec<MenuItem>,
    },
    Separator,
}

pub struct KeyboardShortcut {
    pub key: String,           // "s", "z", "Return", etc.
    pub modifiers: Modifiers,
}

pub struct Modifiers {
    pub cmd: bool,
    pub shift: bool,
    pub alt: bool,
    pub ctrl: bool,
}

impl MenuBar {
    pub fn new() -> Self;
    pub fn add_item(&mut self, item: MenuItem);
    /// Standard macOS menu structure
    pub fn standard() -> Self {
        // File: New (Cmd+N), Open (Cmd+O), Save (Cmd+S), Close (Cmd+W)
        // Edit: Undo (Cmd+Z), Redo (Cmd+Shift+Z), Cut, Copy, Paste
        // View: Zoom In, Zoom Out, Fullscreen
        // Window: Minimize, Zoom, Bring All to Front
        // Help: Search
    }
}
```

### 9.2 — macOS native menu (cvkg-render-native)

Use `objc2` to create `NSMenu`:
```rust
#[cfg(target_os = "macos")]
fn create_native_menu_bar(menu: &MenuBar) {
    unsafe {
        let app = NSApplication::sharedApplication();
        let main_menu = NSMenu::new();
        
        for item in &menu.items {
            let ns_item = NSMenuItem::new();
            ns_item.setTitle(&item.label);
            if let Some(shortcut) = &item.shortcut {
                ns_item.setKeyEquivalent(&shortcut.key[..]);
                // Set modifier mask
            }
            main_menu.addItem(ns_item);
        }
        
        app.setMainMenu(main_menu);
    }
}
```

### 9.3 — Winit integration

Call `create_native_menu_bar` in `ApplicationHandler::new_events` when the app starts. Update menu item states (enabled/disabled) on each frame based on app state.

### 9.4 — Standard keyboard shortcuts

In native renderer, handle:
- Cmd+N → New window (dispatch to app)
- Cmd+O → File open dialog
- Cmd+S → Save document
- Cmd+W → Close window
- Cmd+Q → Quit app
- Cmd+Z → Undo
- Cmd+Shift+Z → Redo
- Cmd+C → Copy
- Cmd+V → Paste
- Cmd+X → Cut
- Cmd+A → Select All
- Cmd+F → Find

**Verification**: Running the app shows a native macOS menu bar with File, Edit, View, Window, Help menus. Shortcuts work.

---

## Item 10: Performance Profiling Overlay

**Status**: ✅ Complete — PerfOverlay component (cvkg-components/src/perf_overlay.rs) with FPS counter, frame time graph, draw call stats. OS-agnostic Cmd+Shift+P toggle.

**Current state**: Telemetry system exists (FPS, frame time, jitter) but no visual overlay.

### 10.1 — Performance overlay component (cvkg-components/src/visual.rs)

```rust
pub struct PerformanceOverlay {
    show_fps: bool,
    show_frame_time: bool,
    show_gpu_time: bool,
    show_draw_calls: bool,
    show_vram: bool,
    show_a11y_tree_size: bool,
    position: OverlayPosition,
    history: Vec<FrameSample>,  // Rolling window of 120 frames
}

pub struct FrameSample {
    pub frame_time_ms: f32,
    pub gpu_time_ms: f32,
    pub draw_calls: u32,
    pub vertices: u32,
    pub vram_mb: u32,
}
```

### 10.2 — Graph rendering

Render miniature graphs (64x32 px) for:
- Frame time over last 120 frames (target: 16.6ms line)
- GPU time over last 120 frames
- VRAM usage

Use `renderer.fill_rect` and `renderer.draw_line` for the graphs.

### 10.3 — Toggle shortcut

In native renderer, handle `Cmd+Shift+P` to toggle the overlay.

### 10.4 — Integration with telemetry

On each frame, the renderer updates telemetry data. The overlay reads it via `renderer.get_telemetry()`.

**Verification**: Press Cmd+Shift+P in the showcase app to see FPS, frame time graph, and VRAM usage.

---

## Item 11: Accessibility Inspector

**Status**: ✅ Complete — A11yInspector + A11yNode (cvkg-components/src/a11y_inspector.rs) with tree viewer, role badges, focus indicators. OS-agnostic Cmd+Shift+I toggle.

**Current state**: AccessKit roles are set but there's no way to inspect the accessibility tree at runtime.

### 11.1 — A11yInspector component (cvkg-components/src/hlin_accessibility.rs)

Enhance existing `HlinAccessibility`:

```rust
pub struct A11yInspector {
    root: Option<AccessKitNode>,
    selected_node: Option<AccessKitNodeId>,
    expanded_nodes: HashSet<AccessKitNodeId>,
    show_invisible: bool,
    filter_role: Option<String>,
}

pub struct AccessKitNode {
    pub id: AccessKitNodeId,
    pub role: String,
    pub label: Option<String>,
    pub value: Option<String>,
    pub bounds: Option<Rect>,
    pub children: Vec<AccessKitNode>,
    pub actions: Vec<String>,
}
```

### 11.2 — Tree renderer

Render the accessibility tree as an expandable list:
```
▼ Button "Submit" [rect: 100,200 120x44]
  ▼ StaticText "Submit"
▼ Checkbox "Enable notifications" [checked]
  ▼ StaticText "Enable notifications"
▼ TextInput "Email address" [focused]
  ▼ StaticText "user@example.com"
```

### 11.3 — Node selection

Click a node in the inspector → highlight the corresponding UI element with a colored border.

### 11.4 — Toggle shortcut

Cmd+Shift+A to toggle the accessibility inspector.

**Verification**: Cmd+Shift+A shows the a11y tree. Clicking a node highlights the UI element.

---

## Item 12: Localization / Internationalization

**Status**: ⬜ Not Started — No i18n pipeline found. Needs fluent-rs or similar integration.

**Current state**: No i18n support. All strings are hardcoded.

### 12.1 — L10n system (cvkg-core)

```rust
pub struct L10n {
    locale: String,
    bundles: HashMap<String, Bundle>,  // lang code → bundle
    current: String,
}

pub struct Bundle {
    strings: HashMap<String, String>,  // key → translated string
    plurals: HashMap<String, PluralRule>,
}

impl L10n {
    pub fn new(locale: &str) -> Self;
    pub fn t(&self, key: &str) -> &str;           // Translate
    pub fn tp(&self, key: &str, count: usize) -> String;  // Translate with plural
    pub fn set_locale(&mut self, locale: &str);
    pub fn current_locale(&self) -> &str;
    pub fn is_rtl(&self) -> bool;
}
```

### 12.2 — String file format

Support `.strings` files (macOS format):
```
"welcome_title" = "Welcome to CVKG";
"item_count" = "%d items";
"item_count_one" = "1 item";
```

### 12.3 — Direction support

Add `Direction` enum to cvkg-core:
```rust
pub enum Direction {
    LTR,
    RTL,
    Auto,
}
```

Propagate through layout → reverse HStack order for RTL.

### 12.4 — Date/number formatting

```rust
pub struct Formatter {
    locale: String,
}

impl Formatter {
    pub fn date(&self, timestamp: f64, style: DateStyle) -> String;
    pub fn number(&self, value: f64, decimals: usize) -> String;
    pub fn currency(&self, value: f64, code: &str) -> String;
    pub fn relative_time(&self, seconds: f64) -> String;  // "2 minutes ago"
}
```

**Verification**: Switch locale in showcase demo → all strings update.

---

## Item 13: Design Token Export

**Status**: ⬜ Not Started — NjordTheme and DesignToken types exist but no export pipeline to JSON/CSS/Swift.

**Current state**: `cvkg-cli theme` command exists but generates a basic struct. No Figma/CSS/Swift export.

### 13.1 — Token export command

Enhance `cvkg tokens export` (already in CLI design):

```bash
cvkg tokens export --format figma --output tokens.json
cvkg tokens export --format css --output variables.css
cvkg tokens export --format swift --output Theme.swift
cvkg tokens export --format json --output tokens.json
```

### 13.2 — Figma format

```json
{
  "colors": {
    "accent": {
      "light": {"r": 0.3, "g": 0.35, "b": 0.75, "a": 1.0},
      "dark": {"r": 1.0, "g": 0.0, "b": 0.4, "a": 1.0}
    }
  },
  "spacing": {"xs": 4, "sm": 8, "md": 16, "lg": 24, "xl": 32},
  "radius": {"sm": 4, "md": 6, "lg": 8, "xl": 12},
  "typography": {"body": 16, "caption": 12, "heading1": 32}
}
```

### 13.3 — CSS format

```css
:root {
  --color-accent-light: rgba(77, 89, 191, 1.0);
  --color-accent-dark: rgba(255, 0, 102, 1.0);
  --spacing-xs: 4px;
  --spacing-sm: 8px;
  --radius-md: 6px;
}
```

### 13.4 — Swift format

Generate SwiftUI-compatible `Color` and `CGFloat` constants.

**Verification**: `cvkg tokens export --format figma` produces valid Figma Tokens JSON.

---

## Item 14: Spatial Audio / Haptic Feedback

**Status**: ⬜ Not Started — No audio or haptic abstraction found.

**Current state**: No audio or haptic feedback system.

### 14.1 — Audio API (cvkg-core)

```rust
pub trait AudioEngine: Send + Sync {
    fn play_sound(&self, name: &str, volume: f32);
    fn play_spatial(&self, name: &str, position: [f32; 3], volume: f32);
    fn set_listener_position(&self, position: [f32; 3]);
}

// Default implementation using rodio or cpal
pub struct SystemAudio;
```

### 14.2 — Haptic API (cvkg-core)

```rust
pub trait HapticEngine: Send + Sync {
    fn impact(&self, intensity: HapticIntensity);
    fn selection(&self);          // Light tap for selection change
    fn success(&self);            // Success notification
    fn warning(&self);            // Warning notification
    fn error(&self);              // Error notification
}

pub enum HapticIntensity {
    Light,
    Medium,
    Heavy,
}

// macOS implementation via Core Haptics
#[cfg(target_os = "macos")]
pub struct MacHapticEngine;
```

### 14.3 — Component integration

- Button click → light haptic impact + click sound
- Toggle on/off → selection haptic
- Error toast → error haptic + error sound
- Success toast → success haptic
- Slider drag → selection haptic on step snap

### 14.4 — Sound design system

Define a set of named sounds:
```rust
pub const SOUND_CLICK: &str = "click";
pub const SOUND_TOGGLE_ON: &str = "toggle_on";
pub const SOUND_TOGGLE_OFF: &str = "toggle_off";
pub const SOUND_SUCCESS: &str = "success";
pub const SOUND_ERROR: &str = "error";
pub const SOUND_SCRUB: &str = "scrub";
```

**Verification**: Clicking buttons produces haptic feedback on Mac trackpad. Toggle sounds play on state change.

---

## Item 15: Scroll Physics and Rubber-Banding

**Current state**: ScrollView component exists but scroll_offset tracking and rubber-band physics are not implemented.

### 15.1 — Scroll physics engine (cvkg-anim)

Extend `SleipnirSolver` for scroll-specific behavior:

```rust
pub struct ScrollPhysics {
    pub position: f32,
    pub velocity: f32,
    pub min: f32,
    pub max: f32,
    pub spring: SleipnirParams,    // For rubber-band
    pub deceleration: f32,         // Pixels/sec² for momentum
}

impl ScrollPhysics {
    pub fn tick(&mut self, dt: f32) -> f32;
    pub fn fling(&mut self, velocity: f32);
    pub fn scroll_to(&mut self, position: f32, animated: bool);
    /// Apply rubber-band resistance when past bounds
    fn apply_rubber_band(&mut self, overscroll: f32) -> f32 {
        overscroll * 0.3 / (1.0 + overscroll.abs() * 0.01)
    }
}
```

### 15.2 — ScrollView rubber-band implementation

In the ScrollView render method:
1. On pointer down: record start position, stop momentum
2. On pointer move: calculate delta, apply to scroll_offset
3. If offset is past content bounds: multiply delta by rubber-band factor
4. On pointer up: calculate velocity from last N positions, start momentum animation
5. On each frame: tick scroll physics, clamp to bounds with rubber-band

### 15.3 — Scroll indicators

Render scroll position indicators:
```rust
fn render_scroll_indicators(
    &self,
    renderer: &mut dyn Renderer,
    viewport_rect: Rect,
    content_size: [f32; 2],
    scroll_offset: [f32; 2],
) {
    let viewport_h = viewport_rect.height;
    let content_h = content_size[1];
    if content_h <= viewport_h { return; }
    
    let scroll_ratio = scroll_offset[1] / (content_h - viewport_h).max(1.0);
    let thumb_height = (viewport_h / content_h * viewport_h).max(20.0);
    let thumb_y = viewport_rect.y + scroll_ratio * (viewport_h - thumb_height);
    
    renderer.fill_rounded_rect(
        Rect { x: viewport_rect.x + viewport_rect.width - 6.0, y: thumb_y, width: 4.0, height: thumb_height },
        2.0,
        theme::text_dim(),  // Semi-transparent
    );
}
```

### 15.4 — Momentum scrolling

Track last 5 pointer positions with timestamps. On release, calculate velocity:
```rust
let velocity = (positions.last().unwrap().pos - positions.first().unwrap().pos)
    / (positions.last().unwrap().time - positions.first().unwrap().time);
physics.fling(velocity);
```

**Verification**: ScrollView with 100 items scrolls smoothly with momentum and rubber-bands at edges.

---

## Item 16: Animation System Integration

**Current state**: `cvkg-anim` has spring physics (SleipnirSolver) but it's not connected to component state changes. Components animate manually with hash IDs.

### 16.1 — Animated modifier (cvkg-core)

```rust
pub struct AnimatedModifier<T: Clone + Send + Sync + 'static> {
    pub target_value: T,
    pub params: SleipnirParams,
    pub interpolator: Arc<dyn Fn(&T, &T, f32) -> T + Send + Sync>,
}

impl<T: Clone + Send + Sync + 'static> ViewModifier for AnimatedModifier<T> {
    fn render_view<V: View>(&self, view: &V, renderer: &mut dyn Renderer, rect: Rect) {
        // Get or create SleipnirSolver from system state
        // Tick solver, apply transform
        // Request redraw if not settled
    }
}
```

### 16.2 — Transition system

```rust
pub enum Transition {
    None,
    Fade { duration: f32 },
    Slide { duration: f32, direction: SlideDirection },
    Scale { duration: f32 },
    Spring(SleipnirParams),
    Custom(Arc<dyn Fn(f32) -> Mat4 + Send + Sync>),
}

// Usage:
// view.transition(Transition::Spring(SleipnirParams::snappy()))
```

### 16.3 — Staggered animations for lists

```rust
pub struct StaggerConfig {
    pub delay_per_item: f32,
    pub direction: StaggerDirection,
    pub params: SleipnirParams,
}

// When adding items to a list, each item animates in with increasing delay
```

### 16.4 — Gesture-driven animations

Connect scroll position to animations:
```rust
// Parallax header: header moves at 0.5x scroll speed
// Pull-to-refresh: resistance increases with pull distance
// Tab switch: content slides with finger position
```

**Verification**: A demo page shows animated list insertion/removal, staggered animations, and gesture-driven transitions.

---

## Item 17: Hot Reload / Dev Server (DEPENDS on CLI)

**Status**: ✅ Complete — FileWatcher (notify 6.0), HotReloadState serialization, ErrorOverlay, WS server integration. `cvkg dev` command with hot reload. OS-agnostic.

**Current state**: `cvkg-cli` has the command structure but hot reload is not implemented. The `dev_runtime`, `ws_server`, and `patch_engine` modules exist as stubs.

### 17.1 — File watcher (cvkg-cli/src/dev_runtime.rs)

Use `notify` crate (already a workspace dependency):
```rust
pub struct FileWatcher {
    watcher: RecommendedWatcher,
    rx: std::sync::mpsc::Receiver<notify::Event>,
    debounce: std::time::Duration,
}

impl FileWatcher {
    pub fn new(paths: Vec<PathBuf>) -> Self;
    /// Returns paths of changed files, debounced
    pub fn poll_changes(&self) -> Vec<PathBuf>;
}
```

### 17.2 — Incremental rebuild

On file change:
1. Run `cargo check` on the changed crate only (fast feedback)
2. If check passes, run `cargo build --lib -p <crate>`
3. Extract the `.dylib`/`.so` from the build output
4. Load the new library via `libloading::Library::new()`
5. Resolve the `extern "C" fn cvkg_main() -> Box<dyn View>` symbol
6. Replace the running view
7. Serialize state to temp file before reload, deserialize after

### 17.3 — State preservation

```rust
pub struct StateSnapshot {
    theme_mode: String,
    window_size: (f32, f32),
    scroll_positions: HashMap<String, [f32; 2]>,
    input_text: HashMap<String, String>,
    // ... serialized component state
}

impl StateSnapshot {
    pub fn save(&self, path: &Path);
    pub fn load(path: &Path) -> Self;
}
```

### 17.4 — Error overlay

If compilation fails, show an error overlay in the app window:
```rust
struct ErrorOverlay {
    message: String,
    file: Option<String>,
    line: Option<u32>,
    column: Option<u32>,
}
```

Render as a red banner at the top of the window with the error message.

### 17.5 — WS server for inspector

`cvkg-cli ws_server` opens a WebSocket connection that:
- Streams telemetry data (FPS, frame time, draw calls) to the Inspector
- Accepts commands: `toggle_overlay`, `set_theme`, `reload`, `get_a11y_tree`

### 17.6 — DevDashboard

`cvkg-cli dashboard` opens a web-based dashboard showing:
- Component tree (live)
- Theme editor (color pickers that update the running app)
- Event log
- Performance graphs

**Verification**: `cvkg dev -p showcase` starts the dev server. Edit a file → app reloads with changes. State is preserved.

---

## Summary of Dependencies

```
#1 (Text Input) ──────► #5 (Real Implementation)
#2 (Layout) ──────────► #3 (Showcase needs layout)
#4 (Undo/Redo) ───────► #1 (Input undo)
                      
#6 (Multi-Window) ────► #3 (Showcase demo)
#7 (Notifications) ──► #6 (Window-level dispatching)
#8 (File Ops) ────────► #6 (Window-level dialogs)
#9 (Menu Bar) ────────► #6 (Window events)
                      
#10 (Perf Overlay) ───► #3 (Showcase demo)
#11 (A11y Inspector) ─► #3 (Showcase demo)
#12 (i18n) ───────────► #3 (Showcase demo)
#13 (Token Export) ───► CLI
#14 (Audio/Haptic) ──► Native renderer
#15 (Scroll Physics) ─► #2 (ScrollView)
#16 (Animation) ──────► #2 (Layout transitions)
#17 (Hot Reload) ─────► CLI + Native renderer
```

## Implementation Order (Respecting Dependencies)

1. **#1 Text Input** — foundational, blocks everything else
2. **#2 Layout** — scrolling, grid, overlay
3. **#5 Real Text Input** — full implementation of #1
4. **#4 Undo/Redo** — depends on Input
5. **#6 Multi-Window** — needed for platform features
6. **#9 Menu Bar** — depends on multi-window
7. **#8 File Operations** — depends on multi-window
8. **#7 Notifications** — depends on multi-window
9. **#14 Audio/Haptic** — platform integration
10. **#15 Scroll Physics** — depends on layout
11. **#16 Animation** — depends on layout
12. **#3 Showcase Demo** — needs all components
13. **#10 Perf Overlay** — showcase integration
14. **#11 A11y Inspector** — showcase integration
15. **#12 i18n** — showcase integration
16. **#13 Token Export** — CLI tooling
17. **#17 Hot Reload** — CLI tooling, final polish
