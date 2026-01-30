// src/grid/cell.rs

use glam::{IVec2, Quat, Vec3};
use std::fmt;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CellType {
    Empty,
    Floor,
    Wall,
}

// Implement Display for CellType first
impl fmt::Display for CellType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let c = match self {
            CellType::Empty => '.',
            CellType::Floor => '#',
            CellType::Wall => 'W',
        };
        write!(f, "{}", c)
    }
}

#[derive(Clone, Copy)]
pub struct LogicCell {
    pub cell_type: CellType,
}

// Implement Display for LogicCell
impl fmt::Display for LogicCell {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Delegate to CellType's Display
        write!(f, "{}", self.cell_type)
    }
}
