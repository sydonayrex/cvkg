# cvkg-macros

**cvkg-macros** provides procedural macros for the CVKG framework, simplifying the definition of views and state.

## Features

- **`#[derive(View)]`**: Automatically implement the `View` trait for structs.
- **`view!` Macro**: A declarative macro for composing complex view hierarchies (experimental).
- **Environment Key Derives**: Easily define custom tokens for the Yggdrasil design system.

## Usage

This crate is usually not consumed directly. Instead, use the macros re-exported by the main `cvkg` crate.
