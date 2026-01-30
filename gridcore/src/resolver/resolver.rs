// src/resolver/resolver.rs
pub trait CoordOps: Copy + Send + Sync {
	/// Offset from a coordinate
	fn offset(&self, dx: i32, dy: i32) -> Self;

	/// Create a coordinate from x, y
	fn from_xy(x: i32, y: i32) -> Self;

	/// Access x and y for iteration
	fn x(&self) -> i32;
	fn y(&self) -> i32;
}

impl CoordOps for IVec2 {
	fn offset(&self, dx: i32, dy: i32) -> Self {
		*self + IVec2::new(dx, dy)
	}

	fn from_xy(x: i32, y: i32) -> Self {
		IVec2::new(x, y)
	}

	fn x(&self) -> i32 { self.x }
	fn y(&self) -> i32 { self.y }
}



use glam::{IVec2, Vec3, Quat};
use crate::grid::cell::{LogicCell};
use crate::{GridBounds, GridRead};
use crate::grid::MeshPlacement;
use crate::resolver::TileRule;

pub trait VisualResolver {
	type Coord;
	type Cell;
	type Grid: GridRead<Coord = Self::Coord, Cell = Self::Cell>;

	fn resolve_all(&self, grid: &Self::Grid) -> Vec<MeshPlacement>;
	fn resolve_cell(&self, grid: &Self::Grid, coord: Self::Coord) -> Vec<MeshPlacement>;
}

pub struct FastResolver<R, C, Cell, G>
where
	R: TileRule<C, Cell> + Sync,
	C: CoordOps + Send + Sync,
	Cell: Send + Sync,
	G: GridRead<Coord = C, Cell = Cell> + GridBounds<Coord = C> + Sync,
{
	pub rule: R,
	pub _phantom: std::marker::PhantomData<(C, Cell, G)>,
}

impl<R, C, Cell, G> VisualResolver for FastResolver<R, C, Cell, G>
where
	R: TileRule<C, Cell> + Sync,
	C: CoordOps + Send + Sync,
	Cell: Send + Sync,
	G: GridRead<Coord = C, Cell = Cell> + GridBounds<Coord = C> + Sync,
{
	type Coord = C;
	type Cell = Cell;
	type Grid = G;

	fn resolve_all(&self, grid: &Self::Grid) -> Vec<MeshPlacement> {
		let min = grid.min();
		let max = grid.max();

		(min.y()..=max.y())
			.flat_map(|y| {
				(min.x()..=max.x()).flat_map(move |x| {
					let corner = C::from_xy(x, y);
					let neighbors = [
						grid.get(corner.offset(-1, -1)),
						grid.get(corner.offset(0, -1)),
						grid.get(corner.offset(-1, 0)),
						grid.get(corner),
					];
					self.rule.compute(corner, neighbors)
				})
			})
			.collect()
	}


	fn resolve_cell(&self, grid: &Self::Grid, coord: C) -> Vec<MeshPlacement> {
		let affected_corners = [
			coord,
			coord.offset(1, 0),
			coord.offset(0, 1),
			coord.offset(1, 1),
		];
		affected_corners.iter().filter_map(|&corner| {
			let neighbors = [
				grid.get(corner.offset(-1, -1)),
				grid.get(corner.offset(0, -1)),
				grid.get(corner.offset(-1, 0)),
				grid.get(corner),
			];
			self.rule.compute(corner, neighbors)
		}).collect()
	}
}