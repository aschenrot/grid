# External Gridcore Prototype Migration

An earlier, separate `gridcore` prototype informed the current API.

| Prototype concept | Current location |
| --- | --- |
| `GridRead`, `GridWrite`, `GridBounds` | `grid_storage` traits |
| Dense 2D iterators and neighbor helpers | `grid_storage::DenseGrid2D` |
| Resolver separating topology rules from storage | `tile_topology::DualGridResolver` |
| Tile classification rules | `tile_topology::TileRule` and descriptors |

Renderer mesh IDs, placement transforms, `Vec3` and `Quat` were left out of core. Adapters can use the neutral descriptors to implement presentation.
