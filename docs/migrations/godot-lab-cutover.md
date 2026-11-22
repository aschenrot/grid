# Godot and World Lab API Removal

[Grid PR #5](https://github.com/aschenrot/grid/pull/5) removed the Godot adapter and World Lab-specific generation APIs from `main` on 22 November 2026. The change is source-incompatible for applications importing them.

## Removed API

- `godot_grid` Cargo package, GDExtension entry point `gdext_rust_init`, `GodotVisualTileResource`, `GodotGridTopologyMapper`, `GodotGridAdapter` and asset mapping.
- `grid_generation` types: `LabChunkCoord`, `LabGenerationSettings`, `LabTerrainCell`, `LabDebugMarker`, `LabGeneratedChunk`, `LabFormationLayer`, `LabFormationBatch`, `CollisionMergeBox`.
- Generation functions `generate_lab_chunk`, `generate_lab_topology_layers`, `build_owned_halo_formation_layer`, `build_owned_halo_formation_layers`, `merge_collision_cells`; constants `LAYER_GROUND`, `LAYER_SOLID`, `LAYER_WATER`, `LAYER_CLIFF`.

Unchanged neutral APIs include `NoiseFillSettings`, `SeededGridGenerator`, `fill_with_noise`, `cellular_automata_step`, `test_pattern`, grid storage, math and tile topology.

## Migrating a consumer

For an existing Godot application, either temporarily pin Grid to `dd05a2c857c9d0410cb375dbebc108bdaa11dc5c` or move the old Godot and lab behavior into an independent application/adapter. The historical revision is no longer maintained. Source remains accessible in Git history.

Neutral Grid APIs support storage and topology in a replacement application, but **do not reproduce the removed lab generator automatically**. Validate any required generated-world behavior separately.

## Verification

The cutover passed [Linux CI](https://github.com/aschenrot/grid/actions/runs/45037826842) at the reviewed revision, including formatting, tests and the seeded CLI demo. Consult the merged PR for the complete diff.
