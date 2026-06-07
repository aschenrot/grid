# Generation Boundary

`grid_generation` is an optional consumer crate for reusable, engine-neutral
grid generation helpers. It is not part of the default workspace members and it
is not a dependency of `grid_math`, `grid_storage`, `tile_topology`, or
`grid_prelude`.

## Current Scope

The crate may own small neutral helpers such as:

- seeded helper settings;
- noise fill over `DenseGrid2D<CellType>`;
- cellular automata steps over grid storage;
- deterministic test patterns.

Those helpers consume grid topology and storage. They do not define world chunk
streaming, Godot realization, Runenwerk product meaning, SDF payloads, or asset
catalogs.

## Godot World Lab Boundary

The current `Crystonix/godot_world_lab` generator stays outside
`grid_generation`.

It is blocked from extraction because it still owns lab-specific behavior:

- chunk/provider diagnostics;
- `GeneratedChunkData` dictionaries;
- debug marker schema;
- wall threshold tuning;
- room and path carving experiments;
- cache/version behavior tied to the lab provider;
- GDScript array representation.

Only a future stable algorithm contract should move here. A stable contract
needs a neutral cell schema, deterministic settings type, host-independent
debug/observation model, and tests proving the output does not depend on Godot
or Runenwerk semantics.

## Non-Goals

`grid_generation` must not own:

- Godot nodes, resources, previews, or scenes;
- `spatial_streaming` lifecycle state;
- chunk providers, async work, cache ownership, or save/load;
- SDF, ECS, renderer resources, product semantics, or procgen graphs;
- tile-kit asset manifests or mesh/material catalogs.

## M14 Decision

Do not move the Godot lab generator into `grid_generation` for M14.

The existing crate remains available as an optional neutral helper crate, while
Godot World Lab keeps proving generation behavior until the reusable contract is
stable enough to extract without dragging lab semantics into `grid`.
