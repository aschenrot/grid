// src/grid/cell.rs

use glam::{IVec2, Quat, Vec3};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CellType {
	Empty,
	Floor,
	Wall,
}

#[derive(Clone, Copy)]
pub struct LogicCell {
	pub cell_type: CellType,
}