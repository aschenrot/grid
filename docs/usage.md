# Grid Usage

Use `grid_prelude` for neutral grid and dual-grid topology. A simple example:

```rust
use grid_prelude::*;
let mut grid = DenseGrid2D::new(4, 4, CellType::Empty);
grid.set(LogicCellCoord::from_xy(1, 1).coord(), CellType::Wall);
let visual = form_visual_tiles(&grid);
assert_eq!(visual.get(VisualCornerCoord::from_xy(1, 1).coord()).unwrap().descriptor().mask().bits(), 0b1000);
```

## Optional procedural generation

```rust
use grid_generation::{fill_with_noise, NoiseFillSettings};
use grid_prelude::*;
let mut grid = DenseGrid2D::new(16, 16, CellType::Empty);
fill_with_noise(&mut grid, NoiseFillSettings::new(0.1, 0.8, 42));
```

This existing helper combines scalar sampling and thresholding; it does **not** expose a scalar output or editable node API. [Grid #2](https://github.com/aschenrot/grid/issues/2) separately specifies sampled scalar noise, explicit thresholding and configurable cellular automata.

## CLI and regression tests

```sh
cargo run --locked -p cli_dual_grid_demo -- --seed 123
cargo run --locked -p cli_dual_grid_demo -- --random-seed
cargo test --locked
cargo test --locked -p grid_generation
cargo test --locked -p tile_topology
```

## Engine adapters

The owner-authorized breaking [Grid #4 cutover](https://github.com/aschenrot/grid/issues/4) removes the `godot_grid` crate and Lab/World Lab APIs. No GDExtension exists in this workspace after the cutover. See [downstream migration instructions](migrations/godot-lab-cutover.md). The Riesel Workbench and optional RunenRender preview are independent consumers; they do not automatically migrate Godot projects.
