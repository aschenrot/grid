# Grid

Rust libraries for grid coordinates and storage, 2D dual-grid tile topology, and optional procedural generation. The core has no game-engine or renderer dependencies.

## Crates

| Crate | Purpose |
| --- | --- |
| `grid_math` | 2D/3D coordinates, sizes, bounds and indexing |
| `grid_storage` | Dense, sparse and chunked storage; local-grid access traits |
| `tile_topology` | Four-bit dual-grid masks, tile descriptors and invalidation |
| `grid_prelude` | Re-exports of the three core crates |
| `grid_generation` | Optional noise fill, seeded generation and cellular automata |

The CLI demo and generation crate are workspace members but are not part of the default test set.

## Example

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

## Verify

```sh
cargo test --locked
cargo test --locked -p grid_generation
cargo run --locked -p cli_dual_grid_demo -- --seed 123
```

See [usage](docs/usage.md), [dual-grid model](docs/dual-grid-model.md), [architecture](docs/architecture.md), and [roadmap](docs/roadmap.md).

The former Godot adapter was removed. Existing consumers can use the [migration guide](docs/migrations/godot-lab-cutover.md).
