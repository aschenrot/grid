use std::fmt::Display;
use crate::grid::grid::{GridRead, GridBounds};
use crate::resolver::resolver::CoordOps;

/// Prints a 2D-like grid to stdout.
/// Works with any grid implementing `GridRead + GridBounds`,
/// for any `Cell` type that implements `Display`.
pub fn print_grid_generic<G, C>(grid: &G)
where
	G: GridRead<Coord = C> + GridBounds<Coord = C>,
	G::Cell: Display,
	C: CoordOps, // CoordOps gives us x(), y(), from_xy()
{
	let min = grid.min();
	let max = grid.max();

	for y in min.y()..=max.y() {
		for x in min.x()..=max.x() {
			let coord = C::from_xy(x, y);

			let val = grid.get(coord);
			match val {
				Some(v) => print!("{} ", v),
				None => print!(". "),
			}
		}
		println!();
	}
}
