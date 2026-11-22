# Grid

Reusable, **engine-neutral** grid storage, dual-grid topology, and optional procedural generation.

## Workspace

The default Cargo workspace builds only:
- `crates/grid_math`
- `crates/grid_storage`
- `crates/tile_topology`
- `crates/grid_prelude`

The optional `crates/grid_generation` and `demos/cli_dual_grid_demo` are separate workspace members. No game engine, ECS, UI, renderer, or adapter dependency is required by the core.

**Breaking change:** the Godot adapter and lab-specific chunk generator were removed in the owner-authorized [Grid #4 clean cutover](https://github.com/aschenrot/grid/issues/4). Existing external Godot integrations are **not automatically migrated**. See [migration notes](docs/migrations/godot-lab-cutover.md).

## Core usage

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

## Commands

```sh
cargo test --locked
cargo test --locked -p grid_generation
cargo test --locked -p tile_topology
cargo run --locked -p cli_dual_grid_demo -- --seed 123
cargo metadata --locked --no-deps
```

The existing `fill_with_noise` and `cellular_automata_step` functions remain unchanged. Proposed bounded sampling, explicit scalar thresholding, and configurable protected-write CA are documented in [the generation boundary](docs/generation-boundary.md) and tracked in [Grid #2](https://github.com/aschenrot/grid/issues/2). Graph nodes, typed ports, execution and GUI integration are responsibilities of consuming applications/platforms such as Riesel, not Grid core.
