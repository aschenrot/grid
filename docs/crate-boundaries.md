# Crate Boundaries

Dependencies run from consumers to the crates they use:

| Crate | Dependencies within Grid |
| --- | --- |
| `grid_math` | None |
| `grid_storage` | `grid_math` |
| `tile_topology` | `grid_math`, `grid_storage` |
| `grid_prelude` | `grid_math`, `grid_storage`, `tile_topology` |
| `grid_generation` (optional) | `grid_math`, `grid_storage`, `tile_topology` |
| `cli_dual_grid_demo` | `grid_prelude`, `grid_generation` |

Default workspace members are math, storage, topology and prelude. Generation and the CLI are tested separately; the prelude does not import procedural generation dependencies.

The core owns data and topology. Rendering, scene resources, execution graphs and host lifecycle belong to consuming applications. Adapters currently live outside this workspace; that location is not a permanent ban on optional integration crates.

## Migration history

Earlier `gridcore` prototypes were split into these crates. For renamed APIs see [repository prototype migration](migrations/current-grid.md) and [external prototype migration](migrations/external-gridcore.md). The removed Godot adapter and lab generation interfaces are documented in the [Godot migration guide](migrations/godot-lab-cutover.md).
