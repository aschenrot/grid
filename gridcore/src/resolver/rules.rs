use glam::{IVec2, Quat, Vec3};
use crate::grid::{CellType, LogicCell, MeshId, MeshPlacement};
use crate::resolver::TileRule;

pub struct MyRule;

impl TileRule<IVec2, LogicCell> for MyRule {
	fn compute(&self, _corner: IVec2, neighbors: [Option<&LogicCell>; 4]) -> Option<MeshPlacement> {
		// Simple example: place a floor mesh if all neighbors are floor
		if neighbors.iter().all(|n| matches!(n, Some(cell) if cell.cell_type == CellType::Floor)) {
			Some(MeshPlacement {
				mesh_id: MeshId(0),
				position: Vec3::ZERO,
				rotation: Quat::IDENTITY,
				mirror_x: false,
				mirror_y: false,
				metadata: 0,
			})
		} else {
			None
		}
	}
}
