use godot::prelude::*;
use grid_storage::{DenseGrid2D, GridRead, GridSize2};
use tile_topology::{
    CellType, DualGridResolver, LogicCellCoord, Occupancy, TileMask4, VisualCornerCoord,
    VisualTile, VisualTileDescriptor, VisualTileKind, VisualTileRotation,
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

pub trait GodotTileAssetResolver {
    fn asset_key_for_descriptor(&self, descriptor: VisualTileDescriptor) -> GodotTileAssetKey;
}

#[derive(Debug, Copy, Clone, Default)]
pub struct DefaultGodotTileAssetResolver;

impl GodotTileAssetResolver for DefaultGodotTileAssetResolver {
    fn asset_key_for_descriptor(&self, descriptor: VisualTileDescriptor) -> GodotTileAssetKey {
        default_asset_key_for_descriptor(descriptor)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GodotTileAssetMapping {
    pub descriptor: VisualTileDescriptor,
    pub asset_key: GodotTileAssetKey,
}

impl GodotTileAssetMapping {
    pub fn new(descriptor: VisualTileDescriptor, asset_key: impl Into<String>) -> Self {
        Self {
            descriptor,
            asset_key: GodotTileAssetKey::new(asset_key),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct GodotTileAssetCatalog {
    mappings: Vec<GodotTileAssetMapping>,
}

impl GodotTileAssetCatalog {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_mapping(
        mut self,
        descriptor: VisualTileDescriptor,
        asset_key: impl Into<String>,
    ) -> Self {
        self.set_mapping(descriptor, asset_key);
        self
    }

    pub fn set_mapping(&mut self, descriptor: VisualTileDescriptor, asset_key: impl Into<String>) {
        let asset_key = GodotTileAssetKey::new(asset_key);
        if let Some(mapping) = self
            .mappings
            .iter_mut()
            .find(|mapping| mapping.descriptor == descriptor)
        {
            mapping.asset_key = asset_key;
            return;
        }

        self.mappings.push(GodotTileAssetMapping {
            descriptor,
            asset_key,
        });
    }

    pub fn mapped_asset_key(&self, descriptor: VisualTileDescriptor) -> Option<&GodotTileAssetKey> {
        self.mappings
            .iter()
            .find(|mapping| mapping.descriptor == descriptor)
            .map(|mapping| &mapping.asset_key)
    }

    pub fn mappings(&self) -> &[GodotTileAssetMapping] {
        &self.mappings
    }
}

impl GodotTileAssetResolver for GodotTileAssetCatalog {
    fn asset_key_for_descriptor(&self, descriptor: VisualTileDescriptor) -> GodotTileAssetKey {
        self.mapped_asset_key(descriptor)
            .cloned()
            .unwrap_or_else(|| default_asset_key_for_descriptor(descriptor))
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

    pub fn from_visual_tile_with_resolver<R>(tile: VisualTile, resolver: &R) -> Self
    where
        R: GodotTileAssetResolver,
    {
        Self::from_descriptor_with_resolver(tile.visual_corner_coord(), tile.descriptor(), resolver)
    }

    pub fn from_descriptor(corner: VisualCornerCoord, descriptor: VisualTileDescriptor) -> Self {
        Self::from_descriptor_with_resolver(corner, descriptor, &DefaultGodotTileAssetResolver)
    }

    pub fn from_descriptor_with_resolver<R>(
        corner: VisualCornerCoord,
        descriptor: VisualTileDescriptor,
        resolver: &R,
    ) -> Self
    where
        R: GodotTileAssetResolver,
    {
        Self {
            corner: visual_corner_to_vector2i(corner),
            asset_key: resolver.asset_key_for_descriptor(descriptor).key,
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

    #[func]
    pub fn visual_tiles_for_logic_grid(
        &self,
        logic_grid: Array<Variant>,
    ) -> Array<Gd<GodotVisualTileResource>> {
        let grid = dense_cell_grid_from_godot(&logic_grid);
        let adapter = GodotGridAdapter;
        let mut resources = Array::<Gd<GodotVisualTileResource>>::new();

        for tile in adapter.map_logic_grid(&grid) {
            resources.push(&resource_from_data(tile));
        }

        resources
    }

    #[func]
    pub fn visual_tiles_for_dirty_logic_cell(
        &self,
        logic_grid: Array<Variant>,
        cell: Vector2i,
    ) -> Array<Gd<GodotVisualTileResource>> {
        let grid = dense_cell_grid_from_godot(&logic_grid);
        let resolver = DualGridResolver::default();
        let mut resources = Array::<Gd<GodotVisualTileResource>>::new();

        for tile in resolver.resolve_logic_cell(&grid, LogicCellCoord::from_xy(cell.x, cell.y)) {
            resources.push(&resource_from_data(GodotVisualTileData::from_visual_tile(
                tile,
            )));
        }

        resources
    }
}

#[derive(Debug, Copy, Clone, Default)]
pub struct GodotGridAdapter;

impl GodotGridAdapter {
    pub fn map_tile(&self, tile: VisualTile) -> GodotVisualTileData {
        GodotVisualTileData::from_visual_tile(tile)
    }

    pub fn map_tile_with_resolver<R>(&self, tile: VisualTile, resolver: &R) -> GodotVisualTileData
    where
        R: GodotTileAssetResolver,
    {
        GodotVisualTileData::from_visual_tile_with_resolver(tile, resolver)
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

    pub fn map_visual_grid_with_resolver<R>(
        &self,
        visual_grid: &DenseGrid2D<VisualTile>,
        resolver: &R,
    ) -> Vec<GodotVisualTileData>
    where
        R: GodotTileAssetResolver,
    {
        visual_grid
            .iter_cells()
            .map(|(_, tile)| self.map_tile_with_resolver(*tile, resolver))
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

    pub fn map_logic_grid_with_resolver<G, R>(
        &self,
        grid: &G,
        resolver: &R,
    ) -> Vec<GodotVisualTileData>
    where
        G: GridRead,
        G::Cell: Occupancy,
        R: GodotTileAssetResolver,
    {
        let visual_grid = form_visual_tiles(grid);
        self.map_visual_grid_with_resolver(&visual_grid, resolver)
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

    pub fn map_dirty_logic_cell_with_resolver<G, R>(
        &self,
        grid: &G,
        cell: LogicCellCoord,
        resolver: &R,
    ) -> [GodotVisualTileData; 4]
    where
        G: GridRead,
        G::Cell: Occupancy,
        R: GodotTileAssetResolver,
    {
        let resolver_core = DualGridResolver::default();
        resolver_core
            .resolve_logic_cell(grid, cell)
            .map(|tile| GodotVisualTileData::from_visual_tile_with_resolver(tile, resolver))
    }

    pub fn affected_visual_corners(&self, cell: LogicCellCoord) -> [Vector2i; 4] {
        affected_visual_corners_for_logic_cell(cell).map(visual_corner_to_vector2i)
    }
}

pub fn visual_corner_to_vector2i(coord: VisualCornerCoord) -> Vector2i {
    Vector2i::new(coord.x(), coord.y())
}

pub fn asset_key_for_descriptor(descriptor: VisualTileDescriptor) -> GodotTileAssetKey {
    default_asset_key_for_descriptor(descriptor)
}

fn default_asset_key_for_descriptor(descriptor: VisualTileDescriptor) -> GodotTileAssetKey {
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

fn dense_cell_grid_from_godot(logic_grid: &Array<Variant>) -> DenseGrid2D<CellType> {
    let height = logic_grid.len();
    let width = logic_grid
        .iter_shared()
        .filter_map(|row| row.try_to::<Array<Variant>>().ok())
        .map(|row| row.len())
        .max()
        .unwrap_or(0);

    DenseGrid2D::from_fn(GridSize2::new(width, height), |coord| {
        logic_grid
            .get(coord.y as usize)
            .and_then(|row| row.try_to::<Array<Variant>>().ok())
            .and_then(|row| row.get(coord.x as usize))
            .map(cell_type_from_variant)
            .unwrap_or(CellType::Empty)
    })
}

fn cell_type_from_variant(value: Variant) -> CellType {
    if let Ok(is_wall) = value.try_to::<bool>() {
        return if is_wall {
            CellType::Wall
        } else {
            CellType::Empty
        };
    }

    if let Ok(occupied) = value.try_to::<i64>() {
        return if occupied == 0 {
            CellType::Empty
        } else {
            CellType::Wall
        };
    }

    CellType::Empty
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
    fn catalog_overrides_descriptor_asset_keys_without_resource_ownership() {
        let corner = VisualTileDescriptor::from_mask_bits(0b0001);
        let edge = VisualTileDescriptor::from_mask_bits(0b0011);
        let catalog = GodotTileAssetCatalog::new().with_mapping(corner, "stone_corner_a");

        assert_eq!(
            catalog.asset_key_for_descriptor(corner).as_str(),
            "stone_corner_a"
        );
        assert_eq!(catalog.asset_key_for_descriptor(edge).as_str(), "edge_0");

        let mapped = GodotVisualTileData::from_descriptor_with_resolver(
            VisualCornerCoord::from_xy(0, 0),
            corner,
            &catalog,
        );
        assert_eq!(mapped.asset_key, "stone_corner_a");
        assert_eq!(mapped.rotation_degrees_cw, 0);
        assert_eq!(mapped.mask, 0b0001);
    }

    #[test]
    fn catalog_replaces_existing_descriptor_mapping() {
        let descriptor = VisualTileDescriptor::from_mask_bits(0b0010);
        let mut catalog = GodotTileAssetCatalog::new();

        catalog.set_mapping(descriptor, "corner_initial");
        catalog.set_mapping(descriptor, "corner_replacement");

        assert_eq!(catalog.mappings().len(), 1);
        assert_eq!(
            catalog.asset_key_for_descriptor(descriptor).as_str(),
            "corner_replacement"
        );
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
