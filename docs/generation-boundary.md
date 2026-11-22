# Generation Boundary

`grid_generation` is an optional **engine-neutral** consumer of `grid_math`, `grid_storage`, and `tile_topology`. None of the core crates depends on it.

## Active generation APIs

- `NoiseFillSettings`, `SeededGridGenerator`
- `fill_with_noise`: current FastNoiseLite sampling/remap and strict threshold to `DenseGrid2D<CellType>`
- `cellular_automata_step`: existing fixed eight-neighbor, out-of-bounds-as-occupied update
- `test_pattern`: deterministic demonstration data

These are the compatibility baseline. The separate [Grid #2 specification](https://github.com/aschenrot/grid/issues/2) owns the prospective finite sampled scalar `Noise2D`, explicit threshold and configurable protected-write CA contracts. They are **not** implemented or approved by the Godot cleanup.

## Owner-authorized Godot/World Lab cutover

The former `adapters/godot_grid` package, `grid_generation::Lab*` types, `generate_lab_chunk` functions, owned-halo formation, layer constants and product-specific collision merge helpers have been **removed in [#4](https://github.com/aschenrot/grid/issues/4)**. The preceding documentation asserted these lab APIs were outside Grid while the actual source still implemented them; this breaking cutover resolves the contradictory boundary.

The old Godot adapter used those APIs. Any *external* World Lab or GDExtension consumer must explicitly migrate or pin a pre-cutover commit. The change does not prove downstream migration or output compatibility. See [migration instructions](migrations/godot-lab-cutover.md).

## Non-goals

This crate must not own Godot nodes, GDScript data or chunk-provider diagnostics; ECS, SDF or game-specific meaning; rendering, editor graphs, capability/provider registries, process management, network transports, GPU resources, world save/load or asset catalogs. Core topology remains authoritative in `tile_topology`.
