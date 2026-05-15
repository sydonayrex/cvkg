# Migration Guide: From Other UI Frameworks to CVKG

## Overview

This guide helps developers migrate from popular UI frameworks to CVKG.

## From SwiftUI (iOS/macOS)

### Basic Concepts Mapping

| SwiftUI | CVKG |
|---------|------|
| `@State` / `@Binding` | `State<T>` |
| `VStack` | `VStack` |
| `HStack` | `HStack` |
| `ZStack` | `ZStack` |
| `Text` | `Text` |
| `Button` | `Button` |
| `.onTapGesture` | `on_click` or handler closure |

### Example Migration

**SwiftUI:**
```swift
struct ContentView: View {
    @State private var name = 