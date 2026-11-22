# Generation Boundary

`grid_generation` is an optional **engine-neutral** consumer of `grid_math`, `grid_storage`, and `tile_topology`. None of the core crates depends on it.

## Active generation APIs

- `NoiseFillSettings`, `SeededGridGenerator`
- `fill_with_noise`: current FastNoiseLite sampling/remap and strict threshold to `DenseGrid2D<CellType>`
- `cellular_automata_step`: existing fixed eight-neighbor, out-of-bounds-as-occupied update
- `test_pattern`: deterministic demonstration data

These are the unchanged compatibility baseline. The proposed [Grid #2](https://github.com/aschenrot/grid/issues/2) bounded APIs live in `grid_generation::bounded`:

- `sample_noise_2d(size, NoiseSample2D, max_output_bytes)` returns `DenseGrid2D<f32>`, with explicit sampling origin, seed, frequency, and pinned legacy algorithm identity.
- `threshold_2d(&samples, threshold, comparison, max_output_bytes)` returns `DenseGrid2D<bool>`, rejecting non-finite values and retaining the strict-greater-than legacy parity option.
- `cellular_automata_step_2d(&input, rules, optional_protected_mask, max_output_bytes)` reads the previous full grid and returns a new occupancy grid. Boundary-read policy is independent of border/mask write protection; the caller chooses iteration and snapshot retention.
- `DenseGrid2D::try_from_fn` and `try_from_vec` offer fallible allocation/shape validation without replacing the infallible compatibility constructors.

Output byte budgets cover the new output buffer, **not** retained past snapshots, CPU time, OS overcommit, or a future transport. The returned sampled field uses zero-origin local coordinates: callers must retain origin and algorithm metadata to interpret/persist results. Sampling with `f32` world positions has precision limitations at very large coordinates; it does not provide universal high-precision world-space continuity.

**Status:** the new interfaces are a proposal subject to Grid #2 specification acceptance and exact-PR-head CI, not yet an accepted published release. They do not provide Riesel nodes or GUI integration.

## Owner-authorized Godot/World Lab cutover

The former `adapters/godot_grid` package, `grid_generation::Lab*` types, `generate_lab_chunk` functions, owned-halo formation, layer constants and product-specific collision merge helpers have been **removed in [#4](https://github.com/aschenrot/grid/issues/4)**. The preceding documentation asserted these lab APIs were outside Grid while the actual source still implemented them; this breaking cutover resolves the contradictory boundary.

The old Godot adapter used those APIs. Any *external* World Lab or GDExtension consumer must explicitly migrate or pin a pre-cutover commit. The change does not prove downstream migration or output compatibility. See [migration instructions](migrations/godot-lab-cutover.md).

## Non-goals

This crate must not own Godot nodes, GDScript data or chunk-provider diagnostics; ECS, SDF or game-specific meaning; rendering, editor graphs, capability/provider registries, process management, network transports, GPU resources, world save/load or asset catalogs. Core topology remains authoritative in `tile_topology`.
