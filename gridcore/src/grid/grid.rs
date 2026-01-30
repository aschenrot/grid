// src/grid/grid.rs

pub trait GridRead {
	type Coord;
	type Cell;

	fn get(&self, coord: Self::Coord) -> Option<&Self::Cell>;
	fn contains(&self, coord: Self::Coord) -> bool;
}

pub trait GridWrite: GridRead {
	fn set(&mut self, coord: Self::Coord, value: Self::Cell) -> bool;
}

pub trait GridBounds {
	type Coord;

	fn min(&self) -> Self::Coord;
	fn max(&self) -> Self::Coord;
}