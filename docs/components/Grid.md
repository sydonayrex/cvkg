# Grid

`cvkg_components::Grid` -- CSS Grid-style layout

## Import

```rust
use cvkg::prelude::*;
```

## Signature

```rust
Grid::new(columns: Vec<GridTrack>, rows: Vec<GridTrack>) -> Grid
```

## Modifiers

| Modifier | Type | Default | Description |
|---|---|---|---|
| `.column_gap(f32)` | `f32` | `0.0` | Gap between columns |
| `.row_gap(f32)` | `f32` | `0.0` | Gap between rows |
| `.gap(f32)` | `f32` | `0.0` | Gap for both columns and rows |
| `.child(V: View + Clone + 'static)` | `View` | - | Add a child view |

## Example

```rust
Grid::new(
    vec![GridTrack::Fr(1.0), GridTrack::Fr(1.0)],
    vec![GridTrack::Auto, GridTrack::Auto],
)
.column_gap(12.0)
.row_gap(8.0)
```

## Notes

- `GridTrack` supports `Fr` (fractional), `Px` (fixed pixels), `Auto`, and `MinContent`
- Children are placed in grid cells using `.grid_placement(row, column)` modifier
