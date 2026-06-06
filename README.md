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
grid.set(LogicCellCoord::from_xy(1, 1).coord(), CellType::Wall);

let visual = form_visual_tiles(&grid);
let tile = visual
    .get(VisualCornerCoord::from_xy(1, 1).coord())
    .unwrap();

assert_eq!(tile.descriptor().mask().bits(), 0b1000);
```

Run the CLI dual-grid demo:

```text
cargo run -p cli_dual_grid_demo
cargo run -p cli_dual_grid_demo -- --seed 123
cargo run -p cli_dual_grid_demo -- --random-seed
```

Run the normal core validation surface:

```text
cargo test
```

Run the Godot adapter check explicitly:

```text
cargo test -p godot_grid
cargo build -p godot_grid
```
