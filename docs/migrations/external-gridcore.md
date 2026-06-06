# Migration Notes: External Gridcore Prototype

The external `/Users/joshua/Projekte/gridcore` repository was used as prototype
input only.

Kept:

- `GridRead`, `GridWrite`, and `GridBounds` trait shape.
- Dense 2D storage iterators and neighbor helpers.
- Resolver idea that separates topology rules from storage.

Renamed or moved:

- `DenseGrid2D` moved to `grid_storage` with `grid_math::GridCoord2`.
- Resolver behavior moved to `tile_topology::DualGridResolver`.
- Tile-rule behavior moved to neutral descriptor creation.

Discarded from core:

- `MeshPlacement`.
- Renderer mesh ids.
- `Vec3` and `Quat` placement data.
- Any assumption that core topology owns engine transforms or assets.
