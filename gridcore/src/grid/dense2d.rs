// src/grid/dense2d.rs

use glam::IVec2;
use crate::grid::grid::{GridBounds, GridRead, GridWrite};

pub struct DenseGrid2D<T> {
	pub width: usize,
	pub height: usize,
	pub cells: Vec<T>,
}

impl<T> DenseGrid2D<T> {
	pub fn new(width: usize, height: usize, default: T) -> Self
	where
		T: Clone,
	{
		Self {
			width,
			height,
			cells: vec![default; width * height],
		}
	}

	fn index_of(&self, coord: IVec2) -> Option<usize> {
		if coord.x < 0 || coord.y < 0 {
			return None;
		}
		let x = coord.x as usize;
		let y = coord.y as usize;
		if x >= self.width || y >= self.height {
			return None;
		}
		Some(y * self.width + x)
	}

	// =======================
	// Iterators
	// =======================
	pub fn iter_cells(&self) -> impl Iterator<Item = (IVec2, &T)> {
		self.cells
			.iter()
			.enumerate()
			.map(move |(idx, cell)| {
				let x = idx % self.width;
				let y = idx / self.width;
				(IVec2::new(x as i32, y as i32), cell)
			})
	}

	pub fn iter_cells_mut(&mut self) -> impl Iterator<Item = (IVec2, &mut T)> {
		let width = self.width;
		self.cells.iter_mut().enumerate().map(move |(idx, cell)| {
			let x = idx % width;
			let y = idx / width;
			(IVec2::new(x as i32, y as i32), cell)
		})
	}

	// =======================
	// Neighbor coordinates
	// =======================
	pub fn neighbors(&self, coord: IVec2) -> Vec<IVec2> {
		let deltas = [
			IVec2::new(-1, -1),
			IVec2::new(0, -1),
			IVec2::new(1, -1),
			IVec2::new(-1, 0),
			IVec2::new(1, 0),
			IVec2::new(-1, 1),
			IVec2::new(0, 1),
			IVec2::new(1, 1),
		];

		deltas
			.iter()
			.map(|d| coord + *d)
			.filter(|c| self.contains(*c))
			.collect()
	}
}

// =======================
// Trait impls
// =======================
impl<T> GridRead for DenseGrid2D<T> {
	type Coord = IVec2;
	type Cell = T;

	fn get(&self, coord: IVec2) -> Option<&T> {
		self.index_of(coord).map(|idx| &self.cells[idx])
	}

	fn contains(&self, coord: IVec2) -> bool {
		self.index_of(coord).is_some()
	}
}

impl<T> GridWrite for DenseGrid2D<T> {
	fn set(&mut self, coord: IVec2, value: T) -> bool {
		if let Some(idx) = self.index_of(coord) {
			self.cells[idx] = value;
			true
		} else {
			false
		}
	}
}

impl<T> GridBounds for DenseGrid2D<T> {
	type Coord = IVec2;

	fn min(&self) -> IVec2 {
		IVec2::new(0, 0)
	}

	fn max(&self) -> IVec2 {
		IVec2::new(self.width as i32 - 1, self.height as i32 - 1)
	}
}