# Usage

Add the prelude for core grid and topology APIs:

```rust
use grid_prelude::*;
```

Create a logic grid, mark cells as walls, then form visual descriptors:

```rust
use grid_prelude::*;

let mut grid = DenseGrid2D::new(4, 4, CellType::Empty);
grid.set(GridCoord2::new(1, 1), CellType::Wall);

let visual = form_visual_tiles(&grid);
let descriptor = visual.get(GridCoord2::new(1, 1)).unwrap();

assert_eq!(descriptor.mask().bits(), 0b1000);
```

Use `grid_generation` only when procedural helpers are needed:

```rust
use grid_generation::{fill_with_noise, NoiseFillSettings};
use grid_prelude::*;

let mut grid = DenseGrid2D::new(16, 16, CellType::Empty);
fill_with_noise(&mut grid, NoiseFillSettings::new(0.1, 0.8, 42));
```
