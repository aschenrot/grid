# Architecture

Grid separates coordinates, containers, dual-grid topology, and optional generation. Current source modules live under `crates/`; the CLI example is under `demos/`.

## Crate responsibilities

- `grid_math`: 2D/3D coordinates, sizes, bounds, indexing, directions and neighbor offsets.
- `grid_storage`: dense, sparse and chunked grids. `GridRead` and `GridWrite` describe **finite, zero-origin 2D** access; world-space sparse/chunked grids expose their own accessors and bounds.
- `tile_topology`: occupancy masks, 16 tile classifications, rotations, visual descriptors and affected-corner invalidation.
- `grid_generation`: optional noise fill, seeded settings, fixed-rule cellular automata and test patterns. It depends on math, storage and topology.
- `grid_prelude`: wildcard re-exports of the math, storage and topology public items. Generation is not included.

Core crates do not require an engine, ECS, renderer, noise or random-number library. `grid_math` offers `glam` conversions behind an optional feature.

## Coordinate conventions

A finite logic grid starts at `(0, 0)`; positive X moves right and positive Y down. `LogicCellCoord` and `VisualCornerCoord` distinguish topology coordinates.

For a `width × height` logic grid, the visual corner grid has `(width + 1) × (height + 1)` samples. Corner `(cx, cy)` reads:

| Mask bit | Logic cell |
| --- | --- |
| 0, northwest | `(cx - 1, cy - 1)` |
| 1, northeast | `(cx, cy - 1)` |
| 2, southwest | `(cx - 1, cy)` |
| 3, southeast | `(cx, cy)` |

Out-of-bounds cells are treated as empty during topology formation. The standard tile rule uses clockwise quarter-turn rotations. A custom rule can supply other descriptors; see the [dual-grid model](dual-grid-model.md).

A logic-cell change at `(x, y)` affects four corners: `(x, y)`, `(x+1, y)`, `(x, y+1)`, `(x+1, y+1)`.

## Block-origin compatibility

Older prototypes named a `2 × 2` sample by its top-left logic cell. Block origin `(x, y)` maps to visual corner `(x+1, y+1)`. The block-origin helper remains a convenience, not a second coordinate system.

**Range limits:** current APIs use signed 32-bit coordinates; extreme arithmetic/bounds behavior is under review in [Grid #9](https://github.com/aschenrot/grid/issues/9). These conventions do not imply every extreme value is presently safe.
