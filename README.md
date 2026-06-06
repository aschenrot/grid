# Grid

Reusable grid storage, dual-grid topology, procedural generation helpers, and a
thin Godot adapter.

## Workspace

The default workspace builds only engine-neutral core crates:

- `crates/grid_math`
- `crates/grid_storage`
- `crates/tile_topology`
- `crates/grid_prelude`

Generation, demos, and adapters are workspace members but are outside
`default-members`.

## Core Usage

```rust
use grid_prelude::*;

let mut grid = DenseGrid2D::new(4, 4, CellType::Empty);
grid.set(GridCoord2::new(1, 1), CellType::Wall);

let visual = form_visual_tiles(&grid);
let tile = visual.get(GridCoord2::new(1, 1)).unwrap();

assert_eq!(tile.descriptor().mask().bits(), 0b1000);
```

Run the CLI dual-grid demo:

```text
cargo run -p cli_dual_grid_demo
```

Run the normal core validation surface:

```text
cargo test --workspace --default-members
```

Run the Godot adapter check explicitly:

```text
cargo test -p godot_grid
```
