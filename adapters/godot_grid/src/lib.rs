use godot::prelude::*;
use grid_generation::{
    CollisionMergeBox, LAYER_CLIFF, LAYER_GROUND, LAYER_SOLID, LAYER_WATER, LabChunkCoord,
    LabGeneratedChunk, LabGenerationSettings, LabTerrainCell, build_owned_halo_formation_layer,
    build_owned_halo_formation_layers, generate_lab_chunk, generate_lab_topology_layers,
    merge_collision_cells,
};
use grid_storage::{DenseGrid2D, GridRead, GridSize2};
use std::collections::BTreeMap;
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

    #[func]
    pub fn generate_lab_chunk_payload(
        &self,
        chunk_coord: Vector3i,
        settings: Dictionary<Variant, Variant>,
    ) -> Dictionary<Variant, Variant> {
        lab_chunk_to_dictionary(generate_lab_chunk(
            lab_chunk_coord_from_vector3i(chunk_coord),
            lab_settings_from_dictionary(&settings),
        ))
    }

    #[func]
    pub fn generate_lab_chunk_record_payload(
        &self,
        chunk_coord: Vector3i,
        settings: Dictionary<Variant, Variant>,
        request_flags: Dictionary<Variant, Variant>,
    ) -> Dictionary<Variant, Variant> {
        lab_chunk_to_dictionary_with_flags(
            generate_lab_chunk(
                lab_chunk_coord_from_vector3i(chunk_coord),
                lab_settings_from_dictionary(&settings),
            ),
            &request_flags,
        )
    }

    #[func]
    pub fn generate_lab_topology_layers_payload(
        &self,
        chunk_coord: Vector3i,
        settings: Dictionary<Variant, Variant>,
    ) -> Dictionary<Variant, Variant> {
        topology_layers_to_dictionary(
            &generate_lab_topology_layers(
                lab_chunk_coord_from_vector3i(chunk_coord),
                lab_settings_from_dictionary(&settings),
            ),
            lab_settings_from_dictionary(&settings).effective_chunk_size_cells(),
        )
    }

    #[func]
    pub fn formation_layer_payload(
        &self,
        chunk_coord: Vector3i,
        settings: Dictionary<Variant, Variant>,
        layer_id: GString,
        layer_grid: Array<Variant>,
    ) -> Dictionary<Variant, Variant> {
        let settings = lab_settings_from_dictionary(&settings);
        let size = settings.effective_chunk_size_cells();
        let grid = flat_u8_grid_from_godot(&layer_grid, size, size);
        let formation = build_owned_halo_formation_layer(
            lab_chunk_coord_from_vector3i(chunk_coord),
            settings,
            layer_id.to_string().as_str(),
            &grid,
        );
        formation_layer_to_dictionary(formation)
    }

    #[func]
    pub fn formation_layers_payload(
        &self,
        chunk_coord: Vector3i,
        settings: Dictionary<Variant, Variant>,
        topology_layers: Dictionary<Variant, Variant>,
        requested_layer_ids: Array<Variant>,
    ) -> Dictionary<Variant, Variant> {
        let settings = lab_settings_from_dictionary(&settings);
        let size = settings.effective_chunk_size_cells();
        let topology_layers = topology_layers_from_dictionary(&topology_layers, size);
        let requested_layer_ids = string_vec_from_array(&requested_layer_ids);
        let batch = build_owned_halo_formation_layers(
            lab_chunk_coord_from_vector3i(chunk_coord),
            settings,
            &topology_layers,
            &requested_layer_ids,
        );
        let mut formation_layers = Dictionary::<Variant, Variant>::new();
        for (layer_id, layer) in batch.layers {
            formation_layers.set(
                layer_id.as_str(),
                &formation_layer_to_dictionary(layer).to_variant(),
            );
        }
        let mut diagnostics = Dictionary::<Variant, Variant>::new();
        diagnostics.set("authority", "native_grid_generation");
        diagnostics.set("formation_mode", "owned_halo_native_batch");
        diagnostics.set("layer_count", formation_layers.len() as i64);
        diagnostics.set("source_chunk_count", batch.source_chunk_coords.len() as i64);
        diagnostics.set(
            "neighbor_generation_count",
            batch.neighbor_generation_count as i64,
        );

        let mut result = Dictionary::<Variant, Variant>::new();
        result.set("product_type", "NativeFormationLayersPayload");
        result.set("formation_layers", &formation_layers.to_variant());
        result.set("diagnostics", &diagnostics.to_variant());
        result
    }

    #[func]
    pub fn visual_bucket_plan_payload(
        &self,
        chunk_coord: Vector3i,
        formation_products: Dictionary<Variant, Variant>,
        visual_layer_specs: Array<Variant>,
    ) -> Dictionary<Variant, Variant> {
        visual_bucket_plan_to_dictionary(chunk_coord, &formation_products, &visual_layer_specs)
    }

    #[func]
    pub fn dirty_visual_bucket_delta_payload(
        &self,
        chunk_coord: Vector3i,
        changed_cells: Array<Vector2i>,
        formation_products: Dictionary<Variant, Variant>,
        visual_layer_specs: Array<Variant>,
    ) -> Dictionary<Variant, Variant> {
        let mut requested_corners = Dictionary::<Variant, Variant>::new();
        for cell in changed_cells.iter_shared() {
            for corner in
                affected_visual_corners_for_logic_cell(LogicCellCoord::from_xy(cell.x, cell.y))
            {
                requested_corners.set(format!("{},{}", corner.x(), corner.y()).as_str(), true);
            }
        }
        visual_bucket_plan_to_dictionary_with_filter(
            chunk_coord,
            &formation_products,
            &visual_layer_specs,
            Some(&requested_corners),
        )
    }

    #[func]
    pub fn merged_collision_boxes_payload(
        &self,
        solid_grid: Array<Variant>,
        water_grid: Array<Variant>,
        collision_policy: Dictionary<Variant, Variant>,
    ) -> Dictionary<Variant, Variant> {
        let height = solid_grid.len();
        let width = grid_width_from_godot(&solid_grid);
        let solid = flat_u8_grid_from_godot(&solid_grid, width, height);
        let water = flat_u8_grid_from_godot(&water_grid, width, height);
        let liquid_blocks_movement = dict_bool(&collision_policy, "liquid_blocks_movement", true);
        let boxes = merge_collision_cells(width, height, &solid, &water, liquid_blocks_movement);
        collision_boxes_to_dictionary(width, height, &solid, &water, liquid_blocks_movement, boxes)
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

fn lab_chunk_coord_from_vector3i(coord: Vector3i) -> LabChunkCoord {
    LabChunkCoord::new(coord.x, coord.y, coord.z)
}

fn lab_settings_from_dictionary(settings: &Dictionary<Variant, Variant>) -> LabGenerationSettings {
    LabGenerationSettings {
        world_seed: dict_i64(settings, "world_seed", 1337),
        generator_version: dict_i64(settings, "generator_version", 2),
        chunk_size_cells: dict_i64(
            settings,
            "effective_chunk_size_cells",
            dict_i64(settings, "chunk_size_cells", 16),
        )
        .max(4) as usize,
        wall_threshold_percent: dict_i64(settings, "wall_threshold_percent", 34) as i32,
        debug_force_chunk_border: dict_bool(settings, "debug_force_chunk_border", false),
        smoothing_passes: dict_i64(settings, "smoothing_passes", 1).max(0) as usize,
        room_attempts: dict_i64(settings, "room_attempts", 3).max(0) as usize,
        room_min_size: dict_i64(settings, "room_min_size", 3).max(2) as usize,
        room_max_size: dict_i64(settings, "room_max_size", 6).max(2) as usize,
        terrain_noise_frequency: dict_f64(settings, "terrain_noise_frequency", 0.065) as f32,
        liquid_noise_frequency: dict_f64(settings, "liquid_noise_frequency", 0.045) as f32,
        solid_noise_frequency: dict_f64(settings, "solid_noise_frequency", 0.09) as f32,
        liquid_threshold_percent: dict_i64(settings, "liquid_threshold_percent", 35) as i32,
        target_walkable_min_percent: dict_i64(settings, "target_walkable_min_percent", 70) as i32,
        target_walkable_max_percent: dict_i64(settings, "target_walkable_max_percent", 80) as i32,
        liquid_blocks_movement: dict_bool(settings, "liquid_blocks_movement", true),
    }
}

fn dict_i64(settings: &Dictionary<Variant, Variant>, key: &str, default_value: i64) -> i64 {
    settings
        .get(key)
        .and_then(|value| value.try_to::<i64>().ok())
        .unwrap_or(default_value)
}

fn dict_f64(settings: &Dictionary<Variant, Variant>, key: &str, default_value: f64) -> f64 {
    settings
        .get(key)
        .and_then(|value| value.try_to::<f64>().ok())
        .unwrap_or(default_value)
}

fn dict_bool(settings: &Dictionary<Variant, Variant>, key: &str, default_value: bool) -> bool {
    settings
        .get(key)
        .and_then(|value| value.try_to::<bool>().ok())
        .unwrap_or(default_value)
}

fn dict_string(settings: &Dictionary<Variant, Variant>, key: &str, default_value: &str) -> String {
    settings
        .get(key)
        .and_then(|value| value.try_to::<GString>().ok())
        .map(|value| value.to_string())
        .unwrap_or_else(|| default_value.to_string())
}

fn dict_vector2i(
    settings: &Dictionary<Variant, Variant>,
    key: &str,
    default_value: Vector2i,
) -> Vector2i {
    settings
        .get(key)
        .and_then(|value| value.try_to::<Vector2i>().ok())
        .unwrap_or(default_value)
}

fn lab_chunk_to_dictionary(chunk: LabGeneratedChunk) -> Dictionary<Variant, Variant> {
    let request_flags = Dictionary::<Variant, Variant>::new();
    lab_chunk_to_dictionary_with_flags(chunk, &request_flags)
}

fn lab_chunk_to_dictionary_with_flags(
    chunk: LabGeneratedChunk,
    request_flags: &Dictionary<Variant, Variant>,
) -> Dictionary<Variant, Variant> {
    let include_terrain_cells = dict_bool(request_flags, "include_terrain_cells", true);
    let include_debug_markers = dict_bool(request_flags, "include_debug_markers", true);
    let include_diagnostics = dict_bool(request_flags, "include_diagnostics", true);
    let requested_topology_layer_ids = request_flags
        .get("topology_layer_ids")
        .and_then(|value| value.try_to::<Array<Variant>>().ok())
        .map(|array| string_vec_from_array(&array))
        .unwrap_or_default();
    let mut dict = Dictionary::<Variant, Variant>::new();
    dict.set("product_type", "NativeGeneratedWorldChunkPayload");
    dict.set(
        "chunk_coord",
        Vector3i::new(chunk.coord.x, chunk.coord.y, chunk.coord.z),
    );
    dict.set("chunk_size_cells", chunk.size as i64);
    if include_terrain_cells {
        dict.set(
            "terrain_cells",
            &terrain_cells_to_array(&chunk.terrain_cells, chunk.size).to_variant(),
        );
    }
    dict.set(
        "topology_layers",
        &topology_layers_to_dictionary_filtered(
            &chunk.topology_layers,
            chunk.size,
            &requested_topology_layer_ids,
        )
        .to_variant(),
    );
    if include_debug_markers {
        dict.set(
            "debug_markers",
            &debug_markers_to_array(&chunk.debug_markers).to_variant(),
        );
    }
    if include_diagnostics {
        dict.set(
            "diagnostics",
            &terrain_diagnostics_to_dictionary(&chunk).to_variant(),
        );
    }
    dict
}

fn topology_layers_to_dictionary(
    topology_layers: &std::collections::BTreeMap<String, Vec<u8>>,
    size: usize,
) -> Dictionary<Variant, Variant> {
    topology_layers_to_dictionary_filtered(topology_layers, size, &[])
}

fn topology_layers_to_dictionary_filtered(
    topology_layers: &std::collections::BTreeMap<String, Vec<u8>>,
    size: usize,
    requested_layer_ids: &[String],
) -> Dictionary<Variant, Variant> {
    let mut dict = Dictionary::<Variant, Variant>::new();
    let default_layer_ids = [
        LAYER_GROUND.to_string(),
        LAYER_WATER.to_string(),
        LAYER_SOLID.to_string(),
        LAYER_CLIFF.to_string(),
    ];
    let layer_ids = if requested_layer_ids.is_empty() {
        default_layer_ids.as_slice()
    } else {
        requested_layer_ids
    };
    for layer_id in layer_ids {
        if let Some(layer) = topology_layers.get(layer_id) {
            dict.set(
                layer_id.as_str(),
                &u8_grid_to_array(layer, size, size).to_variant(),
            );
        }
    }
    dict
}

fn topology_layers_from_dictionary(
    topology_layers: &Dictionary<Variant, Variant>,
    size: usize,
) -> BTreeMap<String, Vec<u8>> {
    let mut result = BTreeMap::new();
    for layer_id in [LAYER_GROUND, LAYER_WATER, LAYER_SOLID, LAYER_CLIFF] {
        if let Some(layer) = topology_layers
            .get(layer_id)
            .and_then(|value| value.try_to::<Array<Variant>>().ok())
        {
            result.insert(
                layer_id.to_string(),
                flat_u8_grid_from_godot(&layer, size, size),
            );
        }
    }
    result
}

fn string_vec_from_array(values: &Array<Variant>) -> Vec<String> {
    values
        .iter_shared()
        .filter_map(|value| value.try_to::<GString>().ok())
        .map(|value| value.to_string())
        .filter(|value| !value.is_empty())
        .collect()
}

fn terrain_cells_to_array(cells: &[LabTerrainCell], size: usize) -> Array<Variant> {
    let mut rows = Array::<Variant>::new();
    for y in 0..size {
        let mut row = Array::<Variant>::new();
        for x in 0..size {
            row.push(&terrain_cell_to_dictionary(cells[y * size + x]).to_variant());
        }
        rows.push(&row.to_variant());
    }
    rows
}

fn terrain_cell_to_dictionary(cell: LabTerrainCell) -> Dictionary<Variant, Variant> {
    let mut dict = Dictionary::<Variant, Variant>::new();
    dict.set("product_type", "LabTerrainCell");
    dict.set("authority", "native_grid_generation");
    dict.set("surface", cell.surface());
    dict.set("solid", cell.solid);
    dict.set("liquid", cell.liquid);
    dict.set("walkable", !cell.solid && !cell.liquid);
    dict.set("height", f64::from(cell.height_percent) / 100.0);
    dict.set("height_percent", i64::from(cell.height_percent));
    dict.set("material", cell.material());
    let mut flags = Array::<Variant>::new();
    if cell.solid {
        flags.push(&"minable".to_variant());
    }
    dict.set("flags", &flags.to_variant());
    dict
}

fn debug_markers_to_array(markers: &[grid_generation::LabDebugMarker]) -> Array<Variant> {
    let mut array = Array::<Variant>::new();
    for marker in markers {
        let mut dict = Dictionary::<Variant, Variant>::new();
        dict.set("type", marker.marker_type);
        if let Some((x, y, width, height)) = marker.rect {
            dict.set("rect_position", Vector2i::new(x, y));
            dict.set("rect_size", Vector2i::new(width, height));
        }
        if let Some((x, y)) = marker.from {
            dict.set("from", Vector2i::new(x, y));
        }
        if let Some((x, y)) = marker.to {
            dict.set("to", Vector2i::new(x, y));
        }
        array.push(&dict.to_variant());
    }
    array
}

fn terrain_diagnostics_to_dictionary(chunk: &LabGeneratedChunk) -> Dictionary<Variant, Variant> {
    let total = chunk.terrain_cells.len();
    let solid = chunk.terrain_cells.iter().filter(|cell| cell.solid).count();
    let liquid = chunk
        .terrain_cells
        .iter()
        .filter(|cell| cell.liquid)
        .count();
    let walkable = chunk
        .terrain_cells
        .iter()
        .filter(|cell| !cell.solid && !cell.liquid)
        .count();
    let mut dict = Dictionary::<Variant, Variant>::new();
    dict.set("authority", "native_grid_generation");
    dict.set("cell_count", total as i64);
    dict.set("walkable_cell_count", walkable as i64);
    dict.set("solid_cell_count", solid as i64);
    dict.set("liquid_cell_count", liquid as i64);
    dict.set("open_percent", percent_i64(walkable, total));
    dict.set("solid_percent", percent_i64(solid, total));
    dict.set("liquid_percent", percent_i64(liquid, total));
    dict.set("logic_grid_alias", "topology_layers.solid");
    dict
}

fn percent_i64(count: usize, total: usize) -> i64 {
    if total == 0 {
        return 0;
    }
    ((count as f64 * 100.0) / total as f64).round() as i64
}

fn formation_layer_to_dictionary(
    layer: grid_generation::LabFormationLayer,
) -> Dictionary<Variant, Variant> {
    let mut dict = Dictionary::<Variant, Variant>::new();
    dict.set("layer_id", layer.layer_id.as_str());
    dict.set(
        "formation_grid",
        &u8_grid_to_array(&layer.formation_grid, layer.size, layer.size).to_variant(),
    );
    dict.set("formation_origin_cell", Vector2i::new(-1, -1));
    dict.set("owned_visual_origin", Vector2i::new(0, 0));
    dict.set(
        "owned_visual_size",
        Vector2i::new(layer.size as i32 - 1, layer.size as i32 - 1),
    );
    dict.set("formation_mode", "owned_halo_native");
    let mut source_chunks = Array::<Variant>::new();
    for coord in layer.source_chunk_coords {
        source_chunks.push(&Vector3i::new(coord.x, coord.y, coord.z).to_variant());
    }
    dict.set("source_chunk_coords", &source_chunks.to_variant());
    dict
}

#[derive(Debug, Clone)]
struct NativeVisualLayerSpec {
    layer_id: String,
    source_topology_layer: String,
    asset_namespace: String,
    material_variant: String,
    height_offset: f64,
}

#[derive(Debug, Clone)]
struct NativeVisualBucketPayload {
    layer_id: String,
    source_topology_layer: String,
    asset_namespace: String,
    material_variant: String,
    source_asset_key: String,
    height_offset: f64,
    corner_x: Vec<i32>,
    corner_y: Vec<i32>,
    rotation_degrees_cw: Vec<i32>,
    mask: Vec<i32>,
}

impl NativeVisualBucketPayload {
    fn new(spec: &NativeVisualLayerSpec, source_asset_key: &str) -> Self {
        Self {
            layer_id: spec.layer_id.clone(),
            source_topology_layer: spec.source_topology_layer.clone(),
            asset_namespace: spec.asset_namespace.clone(),
            material_variant: spec.material_variant.clone(),
            source_asset_key: source_asset_key.to_string(),
            height_offset: spec.height_offset,
            corner_x: Vec::new(),
            corner_y: Vec::new(),
            rotation_degrees_cw: Vec::new(),
            mask: Vec::new(),
        }
    }

    fn push_tile(&mut self, tile: &GodotVisualTileData, local_corner: Vector2i) {
        self.corner_x.push(local_corner.x);
        self.corner_y.push(local_corner.y);
        self.rotation_degrees_cw.push(tile.rotation_degrees_cw);
        self.mask.push(i32::from(tile.mask));
    }

    fn to_dictionary(&self, raw_bucket_key: &str) -> Dictionary<Variant, Variant> {
        let mut dict = Dictionary::<Variant, Variant>::new();
        dict.set("raw_bucket_key", raw_bucket_key);
        dict.set("layer_id", self.layer_id.as_str());
        dict.set("source_topology_layer", self.source_topology_layer.as_str());
        dict.set("asset_namespace", self.asset_namespace.as_str());
        dict.set("material_variant", self.material_variant.as_str());
        dict.set("source_asset_key", self.source_asset_key.as_str());
        dict.set("height_offset", self.height_offset);
        dict.set("corner_x", &packed_i32_array(&self.corner_x).to_variant());
        dict.set("corner_y", &packed_i32_array(&self.corner_y).to_variant());
        dict.set(
            "rotation_degrees_cw",
            &packed_i32_array(&self.rotation_degrees_cw).to_variant(),
        );
        dict.set("mask", &packed_i32_array(&self.mask).to_variant());
        dict.set("instance_count", self.corner_x.len() as i64);
        dict
    }
}

fn visual_bucket_plan_to_dictionary(
    chunk_coord: Vector3i,
    formation_products: &Dictionary<Variant, Variant>,
    visual_layer_specs: &Array<Variant>,
) -> Dictionary<Variant, Variant> {
    visual_bucket_plan_to_dictionary_with_filter(
        chunk_coord,
        formation_products,
        visual_layer_specs,
        None,
    )
}

fn visual_bucket_plan_to_dictionary_with_filter(
    chunk_coord: Vector3i,
    formation_products: &Dictionary<Variant, Variant>,
    visual_layer_specs: &Array<Variant>,
    allowed_local_corners: Option<&Dictionary<Variant, Variant>>,
) -> Dictionary<Variant, Variant> {
    let specs = native_visual_layer_specs(visual_layer_specs);
    let mut visual_layers = Array::<Variant>::new();
    let mut all_visual_tiles = Array::<Variant>::new();
    let mut asset_keys_seen = BTreeMap::<String, bool>::new();
    let mut layer_buckets = Dictionary::<Variant, Variant>::new();
    let mut native_bucket_payloads = BTreeMap::<String, NativeVisualBucketPayload>::new();
    let mut total_skipped_empty = 0;

    for spec in specs {
        let Some(formation) =
            formation_dictionary_for_layer(formation_products, spec.layer_id.as_str())
        else {
            continue;
        };
        let formation_grid = formation
            .get("formation_grid")
            .and_then(|value| value.try_to::<Array<Variant>>().ok())
            .unwrap_or_else(Array::<Variant>::new);
        let formation_origin_cell =
            dict_vector2i(&formation, "formation_origin_cell", Vector2i::new(-1, -1));
        let owned_visual_origin =
            dict_vector2i(&formation, "owned_visual_origin", Vector2i::new(0, 0));
        let owned_visual_size = dict_vector2i(
            &formation,
            "owned_visual_size",
            grid_dimension_vec(&formation_grid),
        );
        let crop_min = Vector2i::new(
            owned_visual_origin.x - formation_origin_cell.x,
            owned_visual_origin.y - formation_origin_cell.y,
        );
        let crop_max = Vector2i::new(
            crop_min.x + owned_visual_size.x,
            crop_min.y + owned_visual_size.y,
        );
        let formation_mode = dict_string(&formation, "formation_mode", "owned_halo_native");
        let visual_grid = form_visual_tiles(&dense_cell_grid_from_godot(&formation_grid));
        let mut layer_tiles = Array::<Variant>::new();
        let mut layer_asset_keys = BTreeMap::<String, bool>::new();
        let mut buckets = Dictionary::<Variant, Variant>::new();
        let mut bounds = empty_bounds_dictionary();
        let mut skipped_empty = 0;
        let mut cropped_visual_corners = 0;
        let mut emitted_out_of_bounds = 0;

        for (_, tile) in visual_grid.iter_cells() {
            let data = GodotVisualTileData::from_visual_tile(*tile);
            let formation_corner = data.corner;
            if !corner_inside_rect(formation_corner, crop_min, crop_max) {
                continue;
            }
            let local_corner = Vector2i::new(
                formation_corner.x + formation_origin_cell.x,
                formation_corner.y + formation_origin_cell.y,
            );
            if let Some(allowed) = allowed_local_corners {
                let key = format!("{},{}", local_corner.x, local_corner.y);
                if allowed.get(key.as_str()).is_none() {
                    continue;
                }
            }
            cropped_visual_corners += 1;
            if !corner_inside_rect(
                local_corner,
                owned_visual_origin,
                Vector2i::new(
                    owned_visual_origin.x + owned_visual_size.x,
                    owned_visual_origin.y + owned_visual_size.y,
                ),
            ) {
                emitted_out_of_bounds += 1;
            }
            if data.is_empty {
                skipped_empty += 1;
                continue;
            }
            let world_corner = Vector2i::new(
                chunk_coord.x * owned_visual_size.x + local_corner.x,
                chunk_coord.z * owned_visual_size.y + local_corner.y,
            );
            let tile_dict = visual_tile_to_dictionary(
                chunk_coord,
                &spec,
                &data,
                local_corner,
                formation_corner,
                world_corner,
                formation_mode.as_str(),
            );
            expand_bounds(&mut bounds, local_corner);
            asset_keys_seen.insert(data.asset_key.clone(), true);
            layer_asset_keys.insert(data.asset_key.clone(), true);
            append_native_bucket_payload(&mut native_bucket_payloads, &spec, &data, local_corner);
            append_bucket_tile(&mut buckets, data.asset_key.as_str(), &tile_dict);
            layer_tiles.push(&tile_dict.to_variant());
            all_visual_tiles.push(&tile_dict.to_variant());
        }

        total_skipped_empty += skipped_empty;
        let layer_asset_keys_array = sorted_keys_array(&layer_asset_keys);
        let mut diagnostics = Dictionary::<Variant, Variant>::new();
        diagnostics.set("is_valid", emitted_out_of_bounds == 0);
        diagnostics.set("layer_id", spec.layer_id.as_str());
        diagnostics.set("source_topology_layer", spec.source_topology_layer.as_str());
        diagnostics.set("formation_mode", formation_mode.as_str());
        diagnostics.set("formation_origin_cell", formation_origin_cell);
        diagnostics.set("owned_visual_origin", owned_visual_origin);
        diagnostics.set("owned_visual_size", owned_visual_size);
        diagnostics.set("cropped_visual_corners", cropped_visual_corners as i64);
        diagnostics.set("non_empty_visual_tiles", layer_tiles.len() as i64);
        diagnostics.set("skipped_empty_visual_tiles", skipped_empty as i64);
        diagnostics.set(
            "emitted_out_of_bounds_corners",
            emitted_out_of_bounds as i64,
        );
        diagnostics.set("asset_key_count", layer_asset_keys.len() as i64);

        let mut layer = Dictionary::<Variant, Variant>::new();
        layer.set("layer_id", spec.layer_id.as_str());
        layer.set("source_topology_layer", spec.source_topology_layer.as_str());
        layer.set("asset_namespace", spec.asset_namespace.as_str());
        layer.set("material_variant", spec.material_variant.as_str());
        layer.set("height_offset", spec.height_offset);
        layer.set("formation_mode", formation_mode.as_str());
        layer.set("formation_origin_cell", formation_origin_cell);
        layer.set("owned_visual_origin", owned_visual_origin);
        layer.set("owned_visual_size", owned_visual_size);
        layer.set("visual_tiles", &layer_tiles.to_variant());
        layer.set("asset_keys", &layer_asset_keys_array.to_variant());
        layer.set("missing_assets", &Array::<Variant>::new().to_variant());
        layer.set("bounds", &bounds.to_variant());
        layer.set("diagnostics", &diagnostics.to_variant());
        layer.set("buckets", &buckets.to_variant());
        layer_buckets.set(spec.layer_id.as_str(), &buckets.to_variant());
        visual_layers.push(&layer.to_variant());
    }

    let asset_keys = sorted_keys_array(&asset_keys_seen);
    let mut diagnostics = Dictionary::<Variant, Variant>::new();
    diagnostics.set("is_valid", true);
    diagnostics.set("authority", "native_grid_visual_planning");
    diagnostics.set("visual_layer_count", visual_layers.len() as i64);
    diagnostics.set("visual_tile_count", all_visual_tiles.len() as i64);
    diagnostics.set("skipped_empty_visual_tiles", total_skipped_empty as i64);
    diagnostics.set("bucket_count", asset_keys.len() as i64);

    let mut result = Dictionary::<Variant, Variant>::new();
    result.set("product_type", "ChunkVisualPlan");
    result.set("chunk_coord", chunk_coord);
    result.set("formation_mode", "layered");
    result.set("visual_layers", &visual_layers.to_variant());
    result.set("visual_tiles", &all_visual_tiles.to_variant());
    result.set("tiles", &all_visual_tiles.to_variant());
    result.set(
        "native_bucket_payloads",
        &native_bucket_payloads_to_array(&native_bucket_payloads).to_variant(),
    );
    result.set("asset_keys", &asset_keys.to_variant());
    result.set("missing_assets", &Array::<Variant>::new().to_variant());
    result.set("bounds", &empty_bounds_dictionary().to_variant());
    result.set("diagnostics", &diagnostics.to_variant());
    result.set("buckets", &layer_buckets.to_variant());
    result
}

fn native_visual_layer_specs(specs: &Array<Variant>) -> Vec<NativeVisualLayerSpec> {
    if specs.is_empty() {
        return vec![
            NativeVisualLayerSpec {
                layer_id: LAYER_GROUND.to_string(),
                source_topology_layer: LAYER_GROUND.to_string(),
                asset_namespace: "ground".to_string(),
                material_variant: "ground".to_string(),
                height_offset: 0.0,
            },
            NativeVisualLayerSpec {
                layer_id: LAYER_WATER.to_string(),
                source_topology_layer: LAYER_WATER.to_string(),
                asset_namespace: "water".to_string(),
                material_variant: "water".to_string(),
                height_offset: -0.04,
            },
            NativeVisualLayerSpec {
                layer_id: LAYER_SOLID.to_string(),
                source_topology_layer: LAYER_SOLID.to_string(),
                asset_namespace: "solid".to_string(),
                material_variant: "solid".to_string(),
                height_offset: 0.14,
            },
        ];
    }

    specs
        .iter_shared()
        .filter_map(|value| value.try_to::<Dictionary<Variant, Variant>>().ok())
        .map(|dict| {
            let layer_id = dict_string(&dict, "layer_id", LAYER_SOLID);
            NativeVisualLayerSpec {
                source_topology_layer: dict_string(
                    &dict,
                    "source_topology_layer",
                    layer_id.as_str(),
                ),
                asset_namespace: dict_string(&dict, "asset_namespace", layer_id.as_str()),
                material_variant: dict_string(&dict, "material_variant", layer_id.as_str()),
                height_offset: dict_f64(&dict, "height_offset", 0.0),
                layer_id,
            }
        })
        .collect()
}

fn formation_dictionary_for_layer(
    formation_products: &Dictionary<Variant, Variant>,
    layer_id: &str,
) -> Option<Dictionary<Variant, Variant>> {
    if let Some(layer) = formation_products
        .get("formation_layers")
        .and_then(|value| value.try_to::<Dictionary<Variant, Variant>>().ok())
        .and_then(|layers| layers.get(layer_id))
        .and_then(|value| value.try_to::<Dictionary<Variant, Variant>>().ok())
    {
        return Some(layer);
    }
    formation_products
        .get("products")
        .and_then(|value| value.try_to::<Dictionary<Variant, Variant>>().ok())
        .and_then(|products| products.get(format!("{}_formation", layer_id).as_str()))
        .and_then(|value| value.try_to::<Dictionary<Variant, Variant>>().ok())
}

fn visual_tile_to_dictionary(
    chunk_coord: Vector3i,
    spec: &NativeVisualLayerSpec,
    tile: &GodotVisualTileData,
    local_corner: Vector2i,
    formation_corner: Vector2i,
    world_corner: Vector2i,
    formation_mode: &str,
) -> Dictionary<Variant, Variant> {
    let mut dict = Dictionary::<Variant, Variant>::new();
    dict.set("chunk_coord", chunk_coord);
    dict.set("layer_id", spec.layer_id.as_str());
    dict.set("source_topology_layer", spec.source_topology_layer.as_str());
    dict.set("asset_namespace", spec.asset_namespace.as_str());
    dict.set("material_variant", spec.material_variant.as_str());
    dict.set("height_offset", spec.height_offset);
    dict.set("corner", local_corner);
    dict.set("asset_key", tile.asset_key.as_str());
    dict.set("rotation_degrees_cw", i64::from(tile.rotation_degrees_cw));
    dict.set(
        "descriptor_rotation_degrees_cw",
        i64::from(tile.rotation_degrees_cw),
    );
    dict.set("catalog_rotation_correction_degrees_cw", 0i64);
    dict.set(
        "effective_rotation_degrees_cw",
        i64::from(tile.rotation_degrees_cw),
    );
    dict.set("canonical_rotation_degrees_cw", 0i64);
    dict.set("catalog_flip_x", false);
    dict.set("catalog_flip_z", false);
    dict.set("mask", i64::from(tile.mask));
    dict.set("is_empty", tile.is_empty);
    dict.set("formation_mode", formation_mode);
    dict.set("formation_corner", formation_corner);
    dict.set("world_corner", world_corner);
    dict
}

fn append_bucket_tile(
    buckets: &mut Dictionary<Variant, Variant>,
    bucket_key: &str,
    tile_dict: &Dictionary<Variant, Variant>,
) {
    let mut bucket = buckets
        .get(bucket_key)
        .and_then(|value| value.try_to::<Array<Variant>>().ok())
        .unwrap_or_else(Array::<Variant>::new);
    bucket.push(&tile_dict.to_variant());
    buckets.set(bucket_key, &bucket.to_variant());
}

fn append_native_bucket_payload(
    buckets: &mut BTreeMap<String, NativeVisualBucketPayload>,
    spec: &NativeVisualLayerSpec,
    tile: &GodotVisualTileData,
    local_corner: Vector2i,
) {
    let raw_bucket_key = format!(
        "{}|{}|{}",
        spec.layer_id.as_str(),
        tile.asset_key.as_str(),
        spec.material_variant.as_str()
    );
    let bucket = buckets
        .entry(raw_bucket_key)
        .or_insert_with(|| NativeVisualBucketPayload::new(spec, tile.asset_key.as_str()));
    bucket.push_tile(tile, local_corner);
}

fn native_bucket_payloads_to_array(
    buckets: &BTreeMap<String, NativeVisualBucketPayload>,
) -> Array<Variant> {
    let mut array = Array::<Variant>::new();
    for (bucket_key, bucket) in buckets {
        array.push(&bucket.to_dictionary(bucket_key.as_str()).to_variant());
    }
    array
}

fn packed_i32_array(values: &[i32]) -> PackedInt32Array {
    let mut array = PackedInt32Array::new();
    for value in values {
        array.push(*value);
    }
    array
}

fn sorted_keys_array(keys: &BTreeMap<String, bool>) -> Array<Variant> {
    let mut array = Array::<Variant>::new();
    for key in keys.keys() {
        array.push(&key.as_str().to_variant());
    }
    array
}

fn empty_bounds_dictionary() -> Dictionary<Variant, Variant> {
    let mut bounds = Dictionary::<Variant, Variant>::new();
    bounds.set("has_tiles", false);
    bounds.set("min_corner", Vector2i::new(0, 0));
    bounds.set("max_corner", Vector2i::new(0, 0));
    bounds
}

fn expand_bounds(bounds: &mut Dictionary<Variant, Variant>, corner: Vector2i) {
    if !dict_bool(bounds, "has_tiles", false) {
        bounds.set("has_tiles", true);
        bounds.set("min_corner", corner);
        bounds.set("max_corner", corner);
        return;
    }
    let min_corner = dict_vector2i(bounds, "min_corner", corner);
    let max_corner = dict_vector2i(bounds, "max_corner", corner);
    bounds.set(
        "min_corner",
        Vector2i::new(min_corner.x.min(corner.x), min_corner.y.min(corner.y)),
    );
    bounds.set(
        "max_corner",
        Vector2i::new(max_corner.x.max(corner.x), max_corner.y.max(corner.y)),
    );
}

fn corner_inside_rect(corner: Vector2i, min_corner: Vector2i, max_exclusive: Vector2i) -> bool {
    corner.x >= min_corner.x
        && corner.y >= min_corner.y
        && corner.x < max_exclusive.x
        && corner.y < max_exclusive.y
}

fn grid_dimension_vec(grid: &Array<Variant>) -> Vector2i {
    let height = grid.len() as i32;
    let width = grid_width_from_godot(grid) as i32;
    Vector2i::new(width, height)
}

fn collision_boxes_to_dictionary(
    width: usize,
    height: usize,
    solid: &[u8],
    water: &[u8],
    liquid_blocks_movement: bool,
    boxes: Vec<CollisionMergeBox>,
) -> Dictionary<Variant, Variant> {
    let mut merged_boxes = Array::<Variant>::new();
    let blocking_cells = solid
        .iter()
        .zip(water.iter().chain(std::iter::repeat(&0)))
        .filter(|(solid, water)| **solid != 0 || (liquid_blocks_movement && **water != 0))
        .count();
    let mut by_reason = Dictionary::<Variant, Variant>::new();
    let mut by_layer = Dictionary::<Variant, Variant>::new();
    let mut shape_count_by_reason = Dictionary::<Variant, Variant>::new();
    let mut shape_count_by_layer = Dictionary::<Variant, Variant>::new();
    for collision_box in &boxes {
        let mut dict = Dictionary::<Variant, Variant>::new();
        dict.set(
            "rect",
            Rect2i::new(
                Vector2i::new(collision_box.x as i32, collision_box.y as i32),
                Vector2i::new(collision_box.width as i32, collision_box.height as i32),
            ),
        );
        dict.set("reason", collision_box.reason);
        dict.set("source_layer", collision_box.source_layer);
        dict.set("cell_count", collision_box.cell_count() as i64);
        merged_boxes.push(&dict.to_variant());
        increment_dictionary_count_by(
            &mut by_reason,
            collision_box.reason,
            collision_box.cell_count() as i64,
        );
        increment_dictionary_count_by(
            &mut by_layer,
            collision_box.source_layer,
            collision_box.cell_count() as i64,
        );
        increment_dictionary_count(&mut shape_count_by_reason, collision_box.reason);
        increment_dictionary_count(&mut shape_count_by_layer, collision_box.source_layer);
    }
    let mut diagnostics = Dictionary::<Variant, Variant>::new();
    diagnostics.set("grid_width", width as i64);
    diagnostics.set("grid_height", height as i64);
    diagnostics.set("liquid_blocks_movement", liquid_blocks_movement);
    diagnostics.set("blocking_cell_count", blocking_cells as i64);
    diagnostics.set("merged_shape_count", boxes.len() as i64);
    diagnostics.set(
        "merge_ratio",
        if blocking_cells == 0 {
            0.0
        } else {
            boxes.len() as f64 / blocking_cells as f64
        },
    );
    diagnostics.set("counts_by_reason", &by_reason.to_variant());
    diagnostics.set("counts_by_layer", &by_layer.to_variant());
    diagnostics.set(
        "shape_counts_by_reason",
        &shape_count_by_reason.to_variant(),
    );
    diagnostics.set("shape_counts_by_layer", &shape_count_by_layer.to_variant());

    let mut result = Dictionary::<Variant, Variant>::new();
    result.set("product_type", "MergedCollisionBoxes");
    result.set("merged_boxes", &merged_boxes.to_variant());
    result.set("diagnostics", &diagnostics.to_variant());
    result
}

fn increment_dictionary_count(dict: &mut Dictionary<Variant, Variant>, key: &str) {
    increment_dictionary_count_by(dict, key, 1);
}

fn increment_dictionary_count_by(dict: &mut Dictionary<Variant, Variant>, key: &str, amount: i64) {
    let next = dict
        .get(key)
        .and_then(|value| value.try_to::<i64>().ok())
        .unwrap_or(0)
        + amount;
    dict.set(key, next);
}

fn u8_grid_to_array(grid: &[u8], width: usize, height: usize) -> Array<Variant> {
    let mut rows = Array::<Variant>::new();
    for y in 0..height {
        let mut row = Array::<Variant>::new();
        for x in 0..width {
            row.push(&i64::from(grid.get(y * width + x).copied().unwrap_or(0)).to_variant());
        }
        rows.push(&row.to_variant());
    }
    rows
}

fn flat_u8_grid_from_godot(grid: &Array<Variant>, width: usize, height: usize) -> Vec<u8> {
    let mut cells = vec![0; width * height];
    for y in 0..height {
        let Some(row_value) = grid.get(y) else {
            continue;
        };
        let Ok(row) = row_value.try_to::<Array<Variant>>() else {
            continue;
        };
        for x in 0..width {
            let value = row
                .get(x)
                .and_then(|cell| cell.try_to::<i64>().ok())
                .unwrap_or(0);
            cells[y * width + x] = if value == 0 { 0 } else { 1 };
        }
    }
    cells
}

fn grid_width_from_godot(grid: &Array<Variant>) -> usize {
    grid.iter_shared()
        .filter_map(|row| row.try_to::<Array<Variant>>().ok())
        .map(|row| row.len())
        .max()
        .unwrap_or(0)
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
            (0b0100, "corner_270"),
            (0b0101, "edge_270"),
            (0b0110, "diagonal_90"),
            (0b0111, "t_180"),
            (0b1000, "corner_180"),
            (0b1001, "diagonal_0"),
            (0b1010, "edge_90"),
            (0b1011, "t_270"),
            (0b1100, "edge_180"),
            (0b1101, "t_90"),
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
        assert_eq!(mapped.asset_key, "corner_180");
        assert_eq!(mapped.rotation_degrees_cw, 180);
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
