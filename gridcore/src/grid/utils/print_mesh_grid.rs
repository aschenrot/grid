use glam::IVec2;
use crate::grid::grid::{GridRead, GridBounds};
use crate::resolver::resolver::CoordOps;
use std::fmt;
use crate::grid::tileset::MeshPlacement;

/// Print a grid of mesh placements
/// Each cell shows the `mesh_id` if a mesh is placed there, or "." otherwise
pub fn print_mesh_grid<C, G>(grid: &G, placements: &[MeshPlacement])
where
	G: GridRead<Coord = C> + GridBounds<Coord = C>,
	C: CoordOps,
{
	let min = grid.min();
	let max = grid.max();

	for y in min.y()..=max.y() {
		for x in min.x()..=max.x() {
			let id = placements
				.iter()
				.find(|p| p.position.x as i32 == x && p.position.y as i32 == y)
				.map(|p| match p.mesh_id.0 {
					1..=12 => format!("{:02}", p.mesh_id.0), // walls
					20 => "F.".to_string(),                  // floor
					30 => "E.".to_string(),                  // empty
					_ => "..".to_string(),
				})
				.unwrap_or("..".to_string());

			print!("{} ", id);
		}
		println!();
	}
}
