use std::collections::HashMap;
use glam::{Quat, Vec3};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct MeshId(pub usize);

#[derive(Clone, Copy)]
pub struct MeshPlacement {
	pub mesh_id: MeshId,   // which Blender mesh to place
	pub position: Vec3,    // visual position (corner offset)
	pub rotation: Quat,    // orientation
	pub mirror_x: bool,    // optional, for mirrored variants
	pub mirror_y: bool,    // optional
	pub metadata: u32,     // optional variant index
}

/// Trait for anything that can act as a tile key
pub trait TileKey: Copy + Eq + std::hash::Hash {}

impl<T: Copy + Eq + std::hash::Hash> TileKey for T {}

/// High-performance precomputed tileset for dual-grid CA / autotiling
pub struct Tileset<T: TileKey> {
	/// Precomputed mapping: tile -> mask -> mesh
	mapping: HashMap<T, Vec<MeshId>>,
	/// Default mesh for tile type
	defaults: HashMap<T, MeshId>,
	/// Number of bits in the mask (4 for dual grid)
	bits: u8,
}

impl<T: TileKey> Tileset<T> {
	/// Create a new tileset with given number of neighbor bits
	pub fn new(bits: u8) -> Self {
		Self {
			mapping: HashMap::new(),
			defaults: HashMap::new(),
			bits,
		}
	}

	/// Set default mesh for a tile type
	pub fn set_default(&mut self, tile: T, mesh: MeshId) {
		self.defaults.insert(tile, mesh);
	}

	/// Build precomputed mask -> mesh mapping for a tile type
	/// `mask_to_mesh` contains (mask, MeshId) pairs
	pub fn build(&mut self, tile: T, mask_to_mesh: &[(u32, MeshId)]) {
		let size = 1 << self.bits; // 2^4 = 16 masks
		let default = *self.defaults.get(&tile).unwrap_or(&MeshId(0));
		let mut vec = vec![default; size];

		for &(mask, mesh) in mask_to_mesh {
			let idx = mask as usize;
			if idx < size {
				vec[idx] = mesh;
			}
		}

		self.mapping.insert(tile, vec);
	}

	/// Lookup mesh for a given tile + mask
	/// Returns default mesh if mask not explicitly mapped
	pub fn get(&self, tile: T, mask: u32) -> MeshId {
		self.mapping
			.get(&tile)
			.and_then(|vec| vec.get(mask as usize))
			.copied()
			.or_else(|| self.defaults.get(&tile).copied())
			.unwrap_or(MeshId(0))
	}
}

/// Builder for ergonomic tileset creation
pub struct PrecomputedTilesetBuilder<T: TileKey> {
	bits: u8,
	defaults: HashMap<T, MeshId>,
	mappings: HashMap<T, Vec<(u32, MeshId)>>,
}

impl<T: TileKey> PrecomputedTilesetBuilder<T> {
	pub fn new(bits: u8) -> Self {
		Self {
			bits,
			defaults: HashMap::new(),
			mappings: HashMap::new(),
		}
	}

	pub fn default(mut self, tile: T, mesh: MeshId) -> Self {
		self.defaults.insert(tile, mesh);
		self
	}

	pub fn mapping(mut self, tile: T, mask: u32, mesh: MeshId) -> Self {
		self.mappings.entry(tile).or_default().push((mask, mesh));
		self
	}

	pub fn build(self) -> Tileset<T> {
		let mut tileset = Tileset::new(self.bits);

		for (tile, mesh) in self.defaults {
			tileset.set_default(tile, mesh);
		}

		for (tile, mask_vec) in self.mappings {
			tileset.build(tile, &mask_vec);
		}

		tileset
	}
}

/// --- Dual Grid Mask Computation ---
/// Each dual grid cell is defined by four logic tiles
pub const TL: u32 = 1 << 0;
pub const TR: u32 = 1 << 1;
pub const BL: u32 = 1 << 2;
pub const BR: u32 = 1 << 3;

/// Compute mask for a dual grid cell from the four surrounding logic tiles
pub fn compute_dual_mask<T: TileKey>(
	logic: &impl Fn(i32, i32) -> Option<T>,
	x: i32,
	y: i32,
	tile: T,
) -> u32 {
	let mut mask = 0;

	if logic(x, y) == Some(tile) { mask |= TL; }
	if logic(x + 1, y) == Some(tile) { mask |= TR; }
	if logic(x, y + 1) == Some(tile) { mask |= BL; }
	if logic(x + 1, y + 1) == Some(tile) { mask |= BR; }

	mask
}

/// Returns rotation in degrees (0, 90, 180, 270) for a mesh in a dual-cell
/// Handles corners, edges, T-junctions.
/// Returns 0 for full or isolated cells.
pub fn dual_cell_rotation(mask: u32) -> u16 {
	match mask {
		// Corners
		BR => 0,
		BL => 90,
		TR => 180,
		TL => 270,

		// Edges: single row/column filled
		TL | TR => 0,        // top edge
		BL | BR => 180,      // bottom edge
		TL | BL => 270,      // left edge
		TR | BR => 90,       // right edge

		// T-junctions: three tiles present
		TL | TR | BL => 270,  // missing BR
		TL | TR | BR => 180,  // missing BL
		TL | BL | BR => 0,    // missing TR
		TR | BL | BR => 90,   // missing TL

		// Full cell or isolated (0 or all 4)
		0 | TL | TR | BL | BR => 0,

		// Any other combination
		_ => 0,
	}
}
