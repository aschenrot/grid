// src/resolver/tile_rule.rs

use glam::{IVec2, Quat, Vec3};
use crate::grid::cell::{LogicCell};
use crate::grid::MeshPlacement;

/// Generic trait for mapping a neighborhood of logic cells to a mesh placement
pub trait TileRule<C, Cell> {
	/// Given the coordinates of a corner and its 4 neighboring cells, return a MeshPlacement
	fn compute(&self, corner: C, neighbors: [Option<&Cell>; 4]) -> Option<MeshPlacement>;
}
