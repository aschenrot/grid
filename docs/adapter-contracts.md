# Adapter Boundary

Grid produces coordinates, grid values, tile masks, descriptors and changed-corner information. An adapter maps those values into a particular engine or renderer without changing the underlying topology.

## Available inputs

- `grid_math`: local coordinates and dimensions
- `grid_storage`: grid containers and accessors
- `tile_topology`: `LogicCellCoord`, `VisualCornerCoord`, `TileMask4`, `VisualTileDescriptor`, `VisualTileKind`, `VisualTileRotation`
- `tile_topology::form_visual_tiles` and `affected_visual_corners_for_logic_cell`

Topology formation computes the mask. The standard tile rule then classifies it and assigns a clockwise quarter-turn rotation. Adapters consume these results; custom rules may construct other descriptors.

Core coordinates use a top-left origin with positive X right and Y down. Rendering coordinates, asset keys, materials and resource lifetimes are adapter responsibilities.

No engine adapter is currently part of this workspace. The removed Godot adapter and its API migration are documented [here](migrations/godot-lab-cutover.md).
