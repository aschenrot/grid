use godot::prelude::Vector2i;
use grid_math::GridCoord2;
use grid_storage::{DenseGrid2D, GridRead};
use tile_topology::{
    DualGridResolver, Occupancy, VisualTile, VisualTileDescriptor, VisualTileKind,
    affected_corners_for_cell, form_visual_tiles,
};

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
        let descriptor = tile.descriptor();

        Self {
            corner: coord_to_vector2i(tile.coord()),
            asset_key: asset_key_for_descriptor(descriptor).key,
            rotation_degrees_cw: descriptor.rotation().degrees_cw() as i32,
            mask: descriptor.mask().bits(),
            is_empty: descriptor.kind() == VisualTileKind::Empty,
        }
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

    pub fn map_dirty_cell<G>(&self, grid: &G, cell: GridCoord2) -> [GodotVisualTileData; 4]
    where
        G: GridRead,
        G::Cell: Occupancy,
    {
        let resolver = DualGridResolver::default();
        resolver
            .resolve_cell(grid, cell)
            .map(GodotVisualTileData::from_visual_tile)
    }

    pub fn affected_corners(&self, cell: GridCoord2) -> [Vector2i; 4] {
        affected_corners_for_cell(cell).map(coord_to_vector2i)
    }
}

pub fn coord_to_vector2i(coord: GridCoord2) -> Vector2i {
    Vector2i::new(coord.x, coord.y)
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

#[cfg(test)]
mod tests {
    use super::*;
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
            adapter.affected_corners(GridCoord2::new(2, 3)),
            [
                Vector2i::new(2, 3),
                Vector2i::new(3, 3),
                Vector2i::new(2, 4),
                Vector2i::new(3, 4),
            ]
        );
    }
}
