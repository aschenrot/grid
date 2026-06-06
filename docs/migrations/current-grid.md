# Migration Notes: Current Repository Prototype

The previous workspace had a monolithic `gridcore` crate and a `demo` binary.
Those APIs are migrated into the split workspace:

| Previous API | New API |
| --- | --- |
| `gridcore::Grid2d` | `grid_storage::DenseGrid2D` |
| `gridcore::CellType` | `tile_topology::CellType` |
| `Grid2d::corner_mask()` | `tile_topology::dual_grid_mask_at()` |
| `Grid2d::mesh_tiles()` | `tile_topology::form_visual_tiles()` |
| `MeshType` | `VisualTileKind` |
| `MeshId` | `VisualTileDescriptor` |
| `Grid2d::fill_with_noise()` | `grid_generation::fill_with_noise()` |
| `demo` package | `cli_dual_grid_demo` package |

The old print helpers were kept demo-local because they are presentation code,
not reusable core behavior.
