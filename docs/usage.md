# Usage

Add the prelude for core grid and topology APIs:

```rust
use grid_prelude::*;
```

Create a logic grid, mark cells as walls, then form visual descriptors:

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

Use `grid_generation` only when procedural helpers are needed:

```rust
use grid_generation::{fill_with_noise, NoiseFillSettings};
use grid_prelude::*;

let mut grid = DenseGrid2D::new(16, 16, CellType::Empty);
fill_with_noise(&mut grid, NoiseFillSettings::new(0.1, 0.8, 42));
```

Run the CLI demo with deterministic or random seeds:

```text
cargo run -p cli_dual_grid_demo
cargo run -p cli_dual_grid_demo -- --seed 123
cargo run -p cli_dual_grid_demo -- --seed random
cargo run -p cli_dual_grid_demo -- --random-seed
```

Build the Godot adapter explicitly when you need the GDExtension dynamic
library:

```text
cargo build -p godot_grid
```
