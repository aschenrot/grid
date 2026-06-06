# Adapter Contracts

Adapters consume neutral core descriptors and produce engine-facing data. They
do not own topology truth.

## Inputs

Adapters may read:

- `grid_math` coordinates and sizes
- `tile_topology::LogicCellCoord`
- `tile_topology::VisualCornerCoord`
- `grid_storage` grid accessors
- `tile_topology::VisualTileDescriptor`
- `tile_topology::VisualTileKind`
- `tile_topology::VisualTileRotation`
- `tile_topology::affected_visual_corners_for_logic_cell()`
- `tile_topology::form_visual_tiles()`

Adapters must not duplicate mask computation, visual tile classification, or
dirty-cell invalidation.

## Asset-Key Mapping

The core descriptor is mapped to an adapter asset key using a policy local to
the adapter. The Godot adapter currently uses stable string keys:

```text
empty
corner_0
corner_90
corner_180
corner_270
edge_0
edge_90
edge_180
edge_270
diagonal_0
diagonal_90
t_0
t_90
t_180
t_270
full
debug
```

Missing assets are adapter-local errors or placeholders. Core descriptors remain
valid even when a Godot scene or resource is not available.

## Placement Mapping

Core coordinates use top-left origin, positive `x` right, positive `y` down.
Godot-facing data maps a visual corner coordinate to `Vector2i(x, y)`.
Rotation is clockwise degrees. Mirroring and variant selection are adapter-local
metadata and must not alter the core descriptor.

## Godot Scope

`adapters/godot_grid` may depend on `godot` and expose Godot-facing value data.
It must not depend on Bevy, Runenwerk, ECS crates, renderer systems, or
procedural generation unless a future adapter-specific demo explicitly needs it.

## Bevy and Runenwerk

Bevy adapter implementation and Runenwerk integration are out of scope for the
current goal.
