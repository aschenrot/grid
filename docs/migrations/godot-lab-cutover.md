# Godot / World Lab removal — breaking Grid cutover

**State:** owner-authorized breaking change in [Grid #4](https://github.com/aschenrot/grid/issues/4). This documents deliberately removed public APIs; it is **not** a claim that external Godot/World Lab consumers have been migrated or independently tested.

## Removed from the Grid workspace

- Cargo package `godot_grid` and its GDExtension-facing classes, export symbol `gdext_rust_init`, Godot mapper, tile asset catalog and other `adapters/godot_grid` functionality.
- Product/lab-specific `grid_generation` exports: `LabChunkCoord`, `LabGenerationSettings`, `LabTerrainCell`, `LabDebugMarker`, `LabGeneratedChunk`, `LabFormationLayer`, `LabFormationBatch`, `CollisionMergeBox`, `generate_lab_chunk`, `generate_lab_topology_layers`, `build_owned_halo_formation_layer`, `build_owned_halo_formation_layers`, `merge_collision_cells`, and `LAYER_GROUND`, `LAYER_SOLID`, `LAYER_WATER`, `LAYER_CLIFF`.
- The unused Godot dependency and its transitive lockfile packages.

**Unchanged:** `grid_math`, `grid_storage`, `tile_topology`, `grid_prelude`, the CLI demo and the legacy neutral `grid_generation::{NoiseFillSettings, SeededGridGenerator, fill_with_noise, cellular_automata_step, test_pattern}` functions and semantics.

## Downstream migration

Existing integrations using the above removed types/functions will **not compile** on the new Grid revision. Choose one:

1. **Temporary pin:** keep using `dd05a2c857c9d0410cb375dbebc108bdaa11dc5c` (the last pre-cutover integration baseline used for this change) until the downstream owner is ready. The historical revision is not maintained as a release.
2. **Independent adapter:** move any genuinely required Godot host/World Lab product behavior into its own application/adapter repository. Import the neutral Grid math, storage and topology crates as needed, and independently verify the old generated-output and presentation requirements; do not presume bit-identical reproduction of removed lab-generation algorithms.
3. **New Riesel consumer:** the Riesel Grid Workbench is a *different* application and is not a drop-in migration target for Godot APIs. Grid supplies neutral algorithms, Riesel owns operation graphs/capabilities/execution, and RunenRender is an optional visualization owner.

The retired source remains recoverable through **Git history**; no redundant backup or long-term compatibility facade is maintained in this repository.

## Release verification

On the exact intended PR/merge candidate:

```sh
cargo fmt --check
cargo test --locked
cargo test --locked -p grid_generation
cargo test --locked -p tile_topology
cargo run --locked -p cli_dual_grid_demo -- --seed 123
cargo metadata --locked --no-deps
```

Review that the active Cargo dependency graph contains no Godot package, and that no active neutral library source references removed `Lab*` symbols. An Actions job failing **before its first test step** is not Rust evidence; preserve the candidate unmerged until a genuinely reviewable test run is available or the delivery process formally handles that exception.
