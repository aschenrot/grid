# Usage

## Grid and topology

```rust
use grid_prelude::*;

let mut grid = DenseGrid2D::new(4, 4, CellType::Empty);
grid.set(LogicCellCoord::from_xy(1, 1).coord(), CellType::Wall);

let visual = form_visual_tiles(&grid);
let corner = visual
    .get(VisualCornerCoord::from_xy(1, 1).coord())
    .unwrap();
assert_eq!(corner.descriptor().mask().bits(), 0b1000);
```

A logic grid of `width × height` produces `(width + 1) × (height + 1)` visual corners. See [mask conventions](dual-grid-model.md).

## Optional generation

```rust
use grid_generation::{fill_with_noise, NoiseFillSettings};
use grid_prelude::*;

let mut grid = DenseGrid2D::new(16, 16, CellType::Empty);
fill_with_noise(&mut grid, NoiseFillSettings::new(0.1, 0.8, 42));
```

`fill_with_noise` currently combines scalar noise sampling and occupancy classification. Separate scalar sampling and configurable CA remain in [Grid #2](https://github.com/aschenrot/grid/issues/2).

## Run

```sh
cargo run --locked -p cli_dual_grid_demo -- --seed 123
cargo run --locked -p cli_dual_grid_demo -- --random-seed
cargo test --locked
cargo test --locked -p grid_generation
```

The old Godot adapter has a [migration guide](migrations/godot-lab-cutover.md).
