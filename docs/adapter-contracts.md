# Downstream Adapter Contract

Grid owns **engine-neutral** coordinates, typed grid values, four-bit dual-grid masks, visual descriptors and invalidation. Engine-specific adapters belong in downstream projects. **No Godot adapter ships in this workspace after the [Grid #4 breaking cutover](https://github.com/aschenrot/grid/issues/4).**

## Authoritative inputs

- `grid_math` coordinate/size semantics (top-left origin, positive X right and Y down)
- `grid_storage` typed grids and access traits
- `tile_topology::{LogicCellCoord, VisualCornerCoord, VisualTileDescriptor, VisualTileKind, VisualTileRotation}`
- `tile_topology::form_visual_tiles`, `dual_grid_mask_at` and `affected_visual_corners_for_logic_cell`

Adapters must not recompute conflicting mask classification, clockwise rotations, logic-cell out-of-bounds values or dirty-cell invalidation. The descriptor remains valid independently of visual asset availability.

## Presentation and host ownership

A downstream renderer/engine may convert descriptors to asset keys, native meshes, scene objects, draw calls or alternative coordinate systems, but it owns those runtime resources, their lifecycle and any presentation thread. Grid does not ship a required window, renderer, UI toolkit, ECS, Godot, Bevy or Riesel API.

## Legacy Godot migration

The previous `godot_grid` package exposed a GDExtension library (`gdext_rust_init`), `GodotVisualTileResource`, `GodotGridTopologyMapper`, `GodotGridAdapter`, catalog/resolver data and Godot World Lab generation payload methods. Those are **removed**, not silently replaced. External callers should pin the last pre-cutover revision while implementing an independent adapter, or migrate to a platform-neutral presentation host. See [migration notes](migrations/godot-lab-cutover.md).

There is no guarantee the old Godot World Lab or any third-party caller has migrated. New Riesel Grid Workbench node authoring/execution belongs to Riesel and its application, not Grid.
