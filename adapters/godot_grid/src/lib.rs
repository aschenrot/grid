use godot::prelude::*;
use grid_storage::{DenseGrid2D, GridRead};
use tile_topology::{
    DualGridResolver, LogicCellCoord, Occupancy, TileMask4, VisualCornerCoord, VisualTile,
    VisualTileDescriptor, VisualTileKind, VisualTileRotation,
    affected_visual_corners_for_logic_cell, form_visual_tiles,
};

struct GodotGridExtension;

#[gdextension]
unsafe impl ExtensionLibrary for GodotGridExtension {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GodotTileAssetKey {
    key: String,
}

impl GodotTileAssetKey {
    pub fn new(key: impl Into<String>) -> Self {
        Self { key: key.into() }
    }

    pub fn as_str(&self) -> &str {
        &self.key
    }
}

#[derive(Debug, Clone)]
pub struct GodotVisualTileData {
    pub corner: Vector2i,
    pub asset_key: String,
    pub rotation_degrees_cw: i32,
    pub mask: u8,
    pub is_empty: bool,
}

impl GodotVisualTileData {
    pub fn from_visual_tile(tile: VisualTile) -> Self {
        Self::from_descriptor(tile.visual_corner_coord(), tile.descriptor())
    }

    pub fn from_descriptor(corner: VisualCornerCoord, descriptor: VisualTileDescriptor) -> Self {
        Self {
            corner: visual_corner_to_vector2i(corner),
            asset_key: asset_key_for_descriptor(descriptor).key,
            rotation_degrees_cw: i32::from(descriptor.rotation().degrees_cw()),
            mask: descriptor.mask().bits(),
            is_empty: descriptor.kind() == VisualTileKind::Empty,
        }
    }
}

#[derive(GodotClass)]
#[class(init, base=Resource)]
pub struct GodotVisualTileResource {
    base: Base<Resource>,
    #[var]
    pub corner: Vector2i,
    #[var]
    pub asset_key: GString,
    #[var]
    pub rotation_degrees_cw: i64,
    #[var]
    pub mask: i64,
    #[var]
    pub is_empty: bool,
}

#[godot_api]
impl GodotVisualTileResource {
    #[func]
    pub fn setup(
        &mut self,
        corner: Vector2i,
        asset_key: GString,
        rotation_degrees_cw: i64,
        mask: i64,
        is_empty: bool,
    ) {
        self.corner = corner;
        self.asset_key = asset_key;
        self.rotation_degrees_cw = rotation_degrees_cw;
        self.mask = mask;
        self.is_empty = is_empty;
    }
}

#[derive(GodotClass)]
#[class(init, base=RefCounted)]
pub struct GodotGridTopologyMapper {
    base: Base<RefCounted>,
}

#[godot_api]
impl GodotGridTopologyMapper {
    #[func]
    pub fn asset_key_for_mask(&self, mask: i64) -> GString {
        GString::from(asset_key_for_descriptor(descriptor_for_godot_mask(mask)).as_str())
    }

    #[func]
    pub fn kind_for_mask(&self, mask: i64) -> GString {
        GString::from(kind_name(descriptor_for_godot_mask(mask).kind()))
    }

    #[func]
    pub fn rotation_degrees_for_mask(&self, mask: i64) -> i64 {
        i64::from(descriptor_for_godot_mask(mask).rotation().degrees_cw())
    }

    #[func]
    pub fn visual_tile_for_mask(&self, corner: Vector2i, mask: i64) -> Gd<GodotVisualTileResource> {
        let corner = VisualCornerCoord::from_xy(corner.x, corner.y);
        let descriptor = descriptor_for_godot_mask(mask);
        resource_from_data(GodotVisualTileData::from_descriptor(corner, descriptor))
    }

    #[func]
    pub fn affected_visual_corners_for_cell(&self, cell: Vector2i) -> Array<Vector2i> {
        let mut corners = Array::<Vector2i>::new();
        for corner in
            affected_visual_corners_for_logic_cell(LogicCellCoord::from_xy(cell.x, cell.y))
        {
            corners.push(visual_corner_to_vector2i(corner));
        }

        corners
    }
}

#[derive(Debug, Copy, Clone, Default)]
pub struct GodotGridAdapter;

impl GodotGridAdapter {
    pub fn map_tile(&self, tile: VisualTile) -> GodotVisualTileData {
        GodotVisualTileData::from_visual_tile(tile)
    }

    pub fn map_visual_grid(
        &self,
        visual_grid: &DenseGrid2D<VisualTile>,
    ) -> Vec<GodotVisualTileData> {
        visual_grid
            .iter_cells()
            .map(|(_, tile)| self.map_tile(*tile))
            .collect()
    }

    pub fn map_logic_grid<G>(&self, grid: &G) -> Vec<GodotVisualTileData>
    where
        G: GridRead,
        G::Cell: Occupancy,
    {
        let visual_grid = form_visual_tiles(grid);
        self.map_visual_grid(&visual_grid)
    }

    pub fn map_dirty_logic_cell<G>(
        &self,
        grid: &G,
        cell: LogicCellCoord,
    ) -> [GodotVisualTileData; 4]
    where
        G: GridRead,
        G::Cell: Occupancy,
    {
        let resolver = DualGridResolver::default();
        resolver
            .resolve_logic_cell(grid, cell)
            .map(GodotVisualTileData::from_visual_tile)
    }

    pub fn affected_visual_corners(&self, cell: LogicCellCoord) -> [Vector2i; 4] {
        affected_visual_corners_for_logic_cell(cell).map(visual_corner_to_vector2i)
    }
}

pub fn visual_corner_to_vector2i(coord: VisualCornerCoord) -> Vector2i {
    Vector2i::new(coord.x(), coord.y())
}

pub fn asset_key_for_descriptor(descriptor: VisualTileDescriptor) -> GodotTileAssetKey {
    let rotation = descriptor.rotation().degrees_cw();
    let key = match descriptor.kind() {
        VisualTileKind::Empty => "empty".to_string(),
        VisualTileKind::Corner => format!("corner_{rotation}"),
        VisualTileKind::Edge => format!("edge_{rotation}"),
        VisualTileKind::T => format!("t_{rotation}"),
        VisualTileKind::Diagonal => format!("diagonal_{rotation}"),
        VisualTileKind::Full => "full".to_string(),
        VisualTileKind::Debug => "debug".to_string(),
    };

    GodotTileAssetKey::new(key)
}

fn descriptor_for_godot_mask(mask: i64) -> VisualTileDescriptor {
    match u8::try_from(mask).ok().and_then(TileMask4::from_bits) {
        Some(mask) => VisualTileDescriptor::from_mask(mask),
        None => VisualTileDescriptor::new(
            TileMask4::from_bits_truncate(mask as u8),
            VisualTileKind::Debug,
            VisualTileRotation::ZERO,
        ),
    }
}

fn kind_name(kind: VisualTileKind) -> &'static str {
    match kind {
        VisualTileKind::Empty => "empty",
        VisualTileKind::Corner => "corner",
        VisualTileKind::Edge => "edge",
        VisualTileKind::T => "t",
        VisualTileKind::Diagonal => "diagonal",
        VisualTileKind::Full => "full",
        VisualTileKind::Debug => "debug",
    }
}

fn resource_from_data(data: GodotVisualTileData) -> Gd<GodotVisualTileResource> {
    let mut resource = GodotVisualTileResource::new_gd();
    {
        let mut tile = resource.bind_mut();
        tile.setup(
            data.corner,
            GString::from(data.asset_key.as_str()),
            i64::from(data.rotation_degrees_cw),
            i64::from(data.mask),
            data.is_empty,
        );
    }

    resource
}

#[cfg(test)]
mod tests {
    use super::*;
    use grid_math::GridCoord2;
    use grid_storage::DenseGrid2D;
    use tile_topology::{CellType, VisualTileDescriptor};

    #[test]
    fn maps_descriptors_to_stable_godot_asset_keys() {
        let expected = [
            (0b0000, "empty"),
            (0b0001, "corner_0"),
            (0b0010, "corner_90"),
            (0b0011, "edge_0"),
            (0b0100, "corner_180"),
            (0b0101, "edge_90"),
            (0b0110, "diagonal_90"),
            (0b0111, "t_90"),
            (0b1000, "corner_270"),
            (0b1001, "diagonal_0"),
            (0b1010, "edge_270"),
            (0b1011, "t_180"),
            (0b1100, "edge_180"),
            (0b1101, "t_270"),
            (0b1110, "t_0"),
            (0b1111, "full"),
        ];

        for (mask, key) in expected {
            let descriptor = VisualTileDescriptor::from_mask_bits(mask);
            assert_eq!(asset_key_for_descriptor(descriptor).as_str(), key);
        }
    }

    #[test]
    fn godot_mask_conversion_uses_debug_for_invalid_masks() {
        let descriptor = descriptor_for_godot_mask(99);

        assert_eq!(descriptor.kind(), VisualTileKind::Debug);
        assert_eq!(asset_key_for_descriptor(descriptor).as_str(), "debug");
    }

    #[test]
    fn maps_visual_tile_to_godot_value_data() {
        let mut grid = DenseGrid2D::new(1, 1, CellType::Empty);
        grid.set(GridCoord2::new(0, 0), CellType::Wall);

        let visual = form_visual_tiles(&grid);
        let tile = *visual.get(GridCoord2::new(0, 0)).unwrap();
        let mapped = GodotVisualTileData::from_visual_tile(tile);

        assert_eq!(mapped.corner, Vector2i::new(0, 0));
        assert_eq!(mapped.asset_key, "corner_270");
        assert_eq!(mapped.rotation_degrees_cw, 270);
        assert_eq!(mapped.mask, 0b1000);
        assert!(!mapped.is_empty);
    }

    #[test]
    fn dirty_cell_mapping_uses_core_invalidation_order() {
        let adapter = GodotGridAdapter;

        assert_eq!(
            adapter.affected_visual_corners(LogicCellCoord::from_xy(2, 3)),
            [
                Vector2i::new(2, 3),
                Vector2i::new(3, 3),
                Vector2i::new(2, 4),
                Vector2i::new(3, 4),
            ]
        );
    }
}
