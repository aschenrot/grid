# Architecture

This repository owns reusable grid math, storage, dual-grid topology, optional
generation helpers, and engine adapters. The source of truth is the split crate
workspace under `crates/`, plus adapter crates under `adapters/`.

## Crate Shape

- `grid_math` owns coordinates, sizes, bounds, directions, neighbor offsets,
  row-major index helpers, and dual-grid coordinate helpers.
- `grid_storage` owns dense storage and common grid access traits. It depends
  only on `grid_math`.
- `tile_topology` owns occupancy, dual-grid mask formation, visual tile
  classification, descriptor creation, and dirty-cell invalidation. It depends
  only on `grid_math` and `grid_storage`.
- `grid_generation` owns optional procedural generation. It may depend on
  `fastnoise-lite` and `rand`.
- `grid_prelude` is a curated re-export crate for normal core usage.

Core crates are engine-neutral. They do not depend on Godot, Bevy, Runenwerk,
ECS, renderers, asset systems, `fastnoise-lite`, `rand`, or required `glam`.
`grid_math` exposes optional `glam` conversions only behind the `glam` feature.

## Frozen Coordinate Contract

The grid uses integer coordinates with origin `(0, 0)` at the top-left logic
cell. Positive `x` moves right. Positive `y` moves down.

`tile_topology` uses semantic coordinate wrappers at the public topology
boundary:

- `LogicCellCoord` names a logic cell.
- `VisualCornerCoord` names a visual corner/intersection.

A logic grid with dimensions `width x height` has a dual visual corner grid with
dimensions `(width + 1) x (height + 1)`. A visual corner coordinate `(cx, cy)`
samples these four logic cells:

- bit 0, northwest: `(cx - 1, cy - 1)`
- bit 1, northeast: `(cx, cy - 1)`
- bit 2, southwest: `(cx - 1, cy)`
- bit 3, southeast: `(cx, cy)`

Out-of-bounds logic cells are empty. Negative coordinates are never contained in
a dense grid and therefore also resolve as empty during mask formation.

## Rotation Contract

Visual tile rotations are clockwise degrees in the set `0`, `90`, `180`, and
`270`. Rotation is descriptor metadata only; core crates do not create meshes,
nodes, transforms, quaternions, ECS bundles, or asset handles.

## Dirty-Cell Invalidation

Changing logic cell `(x, y)` affects exactly four visual corners:

- `(x, y)`
- `(x + 1, y)`
- `(x, y + 1)`
- `(x + 1, y + 1)`

Adapters and downstream consumers use this list to update visual data. They must
not recompute a different invalidation contract.

## Block-Origin Convenience

Some prototypes name a visual sample by the top-left logic cell of a `2 x 2`
block:

```text
(x, y)      (x + 1, y)
(x, y + 1)  (x + 1, y + 1)
```

That is available as a convenience helper, but it is not the core coordinate
contract. Block origin `(x, y)` maps to visual corner `(x + 1, y + 1)`.
