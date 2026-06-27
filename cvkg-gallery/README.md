# cvkg-gallery

A component gallery browser demo application for the CVKG UI framework. It renders a sidebar of available components grouped by category and displays the selected component in a detail panel.

## Overview

`cvkg-gallery` is a binary crate that showcases the components provided by the `cvkg` and `cvkg-components` crates. It is intended as a visual reference and development aid — not as a library.

### Component categories

The gallery includes entries in the following categories:

| Category        | Components                          |
|-----------------|-------------------------------------|
| Forms           | Button, Checkbox, Input, Toggle, Slider, Select |
| Layout          | VStack, HStack, Text                |
| Navigation      | Tabs                                |
| Overlays        | Tooltip                             |
| Data Display    | Progress, Spinner, Badge            |

Selecting a component in the sidebar renders a live preview in the detail panel to the right.

## Dependencies

| Crate              | Source    | Features   |
|--------------------|-----------|------------|
| `cvkg`             | workspace | `native`   |
| `cvkg-components`  | workspace | —          |

## Usage

Run the gallery from the workspace root:

```bash
cargo run -p cvkg-gallery
```

Or from this crate's directory:

```bash
cd cvkg-gallery
cargo run
```

This launches a native window via `cvkg::native::NativeRenderer` with the gallery app.

## Architecture

- **`GalleryEntry`** — a static descriptor for each component (name, category, render function).
- **`GalleryState`** — holds the component catalog and the currently selected index.
- **`GalleryApp`** — implements `View`, producing an `HStack` with a sidebar (`VStack` of buttons grouped by category) and a detail panel that renders the selected component.
- **`main()`** — starts the native renderer with `GalleryApp`.

## Notes

- This crate is not depended on by any other workspace crate.
- Some entries (Select, Tabs, Tooltip) are placeholder text and do not yet render live component previews.
