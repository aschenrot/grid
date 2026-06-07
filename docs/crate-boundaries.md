# Crate Boundaries

## Dependency Direction

```text
grid_math -> grid_storage -> tile_topology -> grid_prelude
                  ^              ^
                  |              |
             grid_generation ----+

godot_grid -> grid_math + grid_storage + tile_topology + godot
cli_dual_grid_demo -> grid_prelude + grid_generation
```

`grid_math` has no required dependencies. `grid_storage` depends only on
`grid_math`. `tile_topology` depends only on `grid_math` and `grid_storage`.
`grid_generation` is a consumer crate and is not part of the core dependency
chain. `grid_prelude` does not re-export generation so normal core users do not
pull procedural dependencies by accident. See `docs/generation-boundary.md` for
the current extraction boundary; the Godot World Lab generator is not moved into
this crate until its reusable contract is stable.

## Kept, Renamed, Moved, Discarded

From the in-repository `gridcore` prototype:

- `Grid2d` storage moved to `grid_storage::DenseGrid2D`.
- `corner_mask()` moved to `tile_topology::dual_grid_mask_at()` with
  `VisualCornerCoord`.
- `mesh_tiles()` moved to `tile_topology::form_visual_tiles()`.
- `MeshType` became `tile_topology::VisualTileKind`.
- `MeshId` became `tile_topology::VisualTileDescriptor`.
- `fill_with_noise()` moved to `grid_generation::fill_with_noise()`.
- Required `glam` coordinates were replaced with `grid_math::GridCoord2`.
- Core print helpers were discarded in favor of demo-local output functions.

From the separate `/Users/joshua/Projekte/gridcore` prototype:

- `GridRead`, `GridWrite`, and `GridBounds` were kept and moved to
  `grid_storage`.
- Dense-grid iterators were kept and moved to `grid_storage::DenseGrid2D`.
- Abstract resolver concepts informed `tile_topology::DualGridResolver` and
  `TileRule`.
- `MeshPlacement`, `Vec3`, `Quat`, renderer mesh ids, and mesh placement
  ownership were discarded from core crates and are adapter concerns.
- Prototype `Tileset` mesh lookup is represented by adapter-local asset-key
  mapping rather than core-owned mesh handles.

## Workspace Policy

Default workspace members are core crates only:

- `crates/grid_math`
- `crates/grid_storage`
- `crates/tile_topology`
- `crates/grid_prelude`

Adapters, demos, and generation are workspace members but stay outside the
default validation surface unless explicitly tested.
