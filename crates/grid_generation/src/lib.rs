use fastnoise_lite::{FastNoiseLite, FractalType, NoiseType};
use grid_math::{ALL_NEIGHBORS_2D, GridCoord2, GridSize2};
use grid_storage::DenseGrid2D;
use rand::{Rng, SeedableRng, rngs::StdRng};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use tile_topology::CellType;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct NoiseFillSettings {
    pub scale: f32,
    pub threshold: f32,
    pub seed: u32,
}

impl NoiseFillSettings {
    pub const fn new(scale: f32, threshold: f32, seed: u32) -> Self {
        Self {
            scale,
            threshold,
            seed,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SeededGridGenerator {
    rng: StdRng,
}

impl SeededGridGenerator {
    pub fn new(seed: u64) -> Self {
        Self {
            rng: StdRng::seed_from_u64(seed),
        }
    }

    pub fn next_seed(&mut self) -> u32 {
        self.rng.random()
    }

    pub fn noise_fill_settings(&mut self, scale: f32, threshold: f32) -> NoiseFillSettings {
        NoiseFillSettings::new(scale, threshold, self.next_seed())
    }
}

pub fn fill_with_noise(grid: &mut DenseGrid2D<CellType>, settings: NoiseFillSettings) {
    let mut noise = FastNoiseLite::new();
    noise.set_seed(Some(i32::from_ne_bytes(settings.seed.to_ne_bytes())));
    noise.set_noise_type(Some(NoiseType::OpenSimplex2));
    noise.set_fractal_type(Some(FractalType::FBm));
    noise.set_fractal_octaves(Some(4));
    noise.set_frequency(Some(settings.scale));

    for (coord, cell) in grid.iter_cells_mut() {
        let (warped_x, warped_y) = noise.domain_warp_2d(coord.x as f32, coord.y as f32);
        let mut value = (noise.get_noise_2d(warped_x, warped_y) + 1.0) / 2.0;
        value = (value * 1.3).clamp(0.0, 1.0);

        *cell = if value > settings.threshold {
            CellType::Wall
        } else {
            CellType::Empty
        };
    }
}

pub fn cellular_automata_step(grid: &DenseGrid2D<CellType>) -> DenseGrid2D<CellType> {
    DenseGrid2D::from_fn(grid.size(), |coord| {
        let wall_neighbors = ALL_NEIGHBORS_2D
            .iter()
            .map(|offset| coord + *offset)
            .filter(|neighbor| {
                grid.get(*neighbor)
                    .is_none_or(tile_topology::Occupancy::is_occupied)
            })
            .count();

        if wall_neighbors >= 5 {
            CellType::Wall
        } else {
            CellType::Empty
        }
    })
}

pub fn test_pattern(width: usize, height: usize) -> DenseGrid2D<CellType> {
    DenseGrid2D::from_fn(GridSize2::new(width, height), |coord| {
        if is_border(coord, width, height) || (coord.x + coord.y) % 5 == 0 {
            CellType::Wall
        } else {
            CellType::Empty
        }
    })
}

fn is_border(coord: GridCoord2, width: usize, height: usize) -> bool {
    let Ok(x) = usize::try_from(coord.x) else {
        return false;
    };
    let Ok(y) = usize::try_from(coord.y) else {
        return false;
    };

    x == 0 || y == 0 || x == width.saturating_sub(1) || y == height.saturating_sub(1)
}

pub const LAYER_GROUND: &str = "ground";
pub const LAYER_SOLID: &str = "solid";
pub const LAYER_WATER: &str = "water";
pub const LAYER_CLIFF: &str = "cliff";

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LabChunkCoord {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl LabChunkCoord {
    pub const fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct LabGenerationSettings {
    pub world_seed: i64,
    pub generator_version: i64,
    pub chunk_size_cells: usize,
    pub wall_threshold_percent: i32,
    pub debug_force_chunk_border: bool,
    pub smoothing_passes: usize,
    pub room_attempts: usize,
    pub room_min_size: usize,
    pub room_max_size: usize,
    pub terrain_noise_frequency: f32,
    pub liquid_noise_frequency: f32,
    pub solid_noise_frequency: f32,
    pub liquid_threshold_percent: i32,
    pub target_walkable_min_percent: i32,
    pub target_walkable_max_percent: i32,
    pub liquid_blocks_movement: bool,
}

impl Default for LabGenerationSettings {
    fn default() -> Self {
        Self {
            world_seed: 1337,
            generator_version: 2,
            chunk_size_cells: 16,
            wall_threshold_percent: 34,
            debug_force_chunk_border: false,
            smoothing_passes: 1,
            room_attempts: 3,
            room_min_size: 3,
            room_max_size: 6,
            terrain_noise_frequency: 0.065,
            liquid_noise_frequency: 0.045,
            solid_noise_frequency: 0.09,
            liquid_threshold_percent: 35,
            target_walkable_min_percent: 70,
            target_walkable_max_percent: 80,
            liquid_blocks_movement: true,
        }
    }
}

impl LabGenerationSettings {
    pub fn effective_chunk_size_cells(self) -> usize {
        self.chunk_size_cells.max(4)
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct LabTerrainCell {
    pub height_percent: i32,
    pub liquid: bool,
    pub solid: bool,
}

impl LabTerrainCell {
    pub fn walkable(self, settings: LabGenerationSettings) -> bool {
        !self.solid && !(settings.liquid_blocks_movement && self.liquid)
    }

    pub fn surface(self) -> &'static str {
        if self.liquid { "water" } else { "ground" }
    }

    pub fn material(self) -> &'static str {
        if self.liquid {
            "water"
        } else if self.solid {
            "rock"
        } else {
            "ground"
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabDebugMarker {
    pub marker_type: &'static str,
    pub from: Option<(i32, i32)>,
    pub to: Option<(i32, i32)>,
    pub rect: Option<(i32, i32, i32, i32)>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabGeneratedChunk {
    pub coord: LabChunkCoord,
    pub size: usize,
    pub terrain_cells: Vec<LabTerrainCell>,
    pub topology_layers: BTreeMap<String, Vec<u8>>,
    pub debug_markers: Vec<LabDebugMarker>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LabFormationLayer {
    pub layer_id: String,
    pub size: usize,
    pub formation_grid: Vec<u8>,
    pub source_chunk_coords: Vec<LabChunkCoord>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LabFormationBatch {
    pub layers: BTreeMap<String, LabFormationLayer>,
    pub source_chunk_coords: Vec<LabChunkCoord>,
    pub neighbor_generation_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CollisionMergeBox {
    pub x: usize,
    pub y: usize,
    pub width: usize,
    pub height: usize,
    pub reason: &'static str,
    pub source_layer: &'static str,
}

impl CollisionMergeBox {
    pub fn cell_count(&self) -> usize {
        self.width * self.height
    }
}

pub fn generate_lab_chunk(
    coord: LabChunkCoord,
    settings: LabGenerationSettings,
) -> LabGeneratedChunk {
    let size = settings.effective_chunk_size_cells();
    let solid_layer = generate_smoothed_solid_layer(coord, settings);
    let mut terrain_cells = generate_base_terrain_cells(coord, settings, &solid_layer);
    let mut debug_markers = Vec::new();

    carve_rooms_and_paths(coord, settings, &mut terrain_cells, &mut debug_markers);
    if should_enforce_walkable_target(settings) {
        repair_walkable_connectivity(settings, &mut terrain_cells, &mut debug_markers);
        balance_walkable_percent(settings, &mut terrain_cells, &mut debug_markers);
    }
    if settings.debug_force_chunk_border {
        apply_forced_chunk_border(&mut terrain_cells, size);
    }

    LabGeneratedChunk {
        coord,
        size,
        topology_layers: derive_topology_layers(&terrain_cells, size),
        terrain_cells,
        debug_markers,
    }
}

pub fn generate_lab_topology_layers(
    coord: LabChunkCoord,
    settings: LabGenerationSettings,
) -> BTreeMap<String, Vec<u8>> {
    generate_lab_chunk(coord, settings).topology_layers
}

pub fn build_owned_halo_formation_layer(
    coord: LabChunkCoord,
    settings: LabGenerationSettings,
    layer_id: &str,
    current_layer_grid: &[u8],
) -> LabFormationLayer {
    let current_topology_layers =
        BTreeMap::from([(layer_id.to_string(), current_layer_grid.to_vec())]);
    let requested_layer_ids = vec![layer_id.to_string()];
    build_owned_halo_formation_layers(
        coord,
        settings,
        &current_topology_layers,
        &requested_layer_ids,
    )
    .layers
    .remove(layer_id)
    .unwrap_or_else(|| LabFormationLayer {
        layer_id: layer_id.to_string(),
        size: settings.effective_chunk_size_cells() + 1,
        formation_grid: vec![
            0;
            (settings.effective_chunk_size_cells() + 1)
                * (settings.effective_chunk_size_cells() + 1)
        ],
        source_chunk_coords: vec![coord],
    })
}

pub fn build_owned_halo_formation_layers(
    coord: LabChunkCoord,
    settings: LabGenerationSettings,
    current_topology_layers: &BTreeMap<String, Vec<u8>>,
    requested_layer_ids: &[String],
) -> LabFormationBatch {
    let size = settings.effective_chunk_size_cells();
    let formation_size = size + 1;
    let mut requested_layers = BTreeSet::new();
    for layer_id in requested_layer_ids {
        if !layer_id.is_empty() {
            requested_layers.insert(layer_id.clone());
        }
    }
    if requested_layers.is_empty() {
        requested_layers.insert(LAYER_SOLID.to_string());
    }
    let mut formation_grids: BTreeMap<String, Vec<u8>> = requested_layers
        .iter()
        .map(|layer_id| (layer_id.clone(), vec![0; formation_size * formation_size]))
        .collect();
    let mut source_chunks = BTreeSet::new();
    let mut neighbor_topologies: BTreeMap<LabChunkCoord, BTreeMap<String, Vec<u8>>> =
        BTreeMap::new();
    let mut neighbor_generation_count = 0;

    for fy in 0..formation_size {
        for fx in 0..formation_size {
            let local_x = fx as i32 - 1;
            let local_y = fy as i32 - 1;
            let world_x = coord.x * size as i32 + local_x;
            let world_y = coord.z * size as i32 + local_y;
            let owner = LabChunkCoord::new(
                floor_div(world_x, size as i32),
                coord.y,
                floor_div(world_y, size as i32),
            );
            source_chunks.insert(owner);
            let owner_local_x = positive_mod(world_x, size as i32) as usize;
            let owner_local_y = positive_mod(world_y, size as i32) as usize;

            if owner != coord && !neighbor_topologies.contains_key(&owner) {
                neighbor_topologies.insert(owner, generate_lab_topology_layers(owner, settings));
                neighbor_generation_count += 1;
            }

            for layer_id in &requested_layers {
                let value = if owner == coord {
                    topology_layer_grid_value(
                        current_topology_layers,
                        layer_id,
                        size,
                        owner_local_x,
                        owner_local_y,
                    )
                } else {
                    let topology_layers =
                        neighbor_topologies.get(&owner).expect("neighbor inserted");
                    topology_layer_grid_value(
                        topology_layers,
                        layer_id,
                        size,
                        owner_local_x,
                        owner_local_y,
                    )
                };
                if let Some(formation_grid) = formation_grids.get_mut(layer_id) {
                    formation_grid[fy * formation_size + fx] = value;
                }
            }
        }
    }

    let source_chunk_coords: Vec<LabChunkCoord> = source_chunks.into_iter().collect();
    let layers = formation_grids
        .into_iter()
        .map(|(layer_id, formation_grid)| {
            (
                layer_id.clone(),
                LabFormationLayer {
                    layer_id,
                    size: formation_size,
                    formation_grid,
                    source_chunk_coords: source_chunk_coords.clone(),
                },
            )
        })
        .collect();

    LabFormationBatch {
        layers,
        source_chunk_coords,
        neighbor_generation_count,
    }
}

pub fn merge_collision_cells(
    width: usize,
    height: usize,
    solid_grid: &[u8],
    water_grid: &[u8],
    liquid_blocks_movement: bool,
) -> Vec<CollisionMergeBox> {
    let mut groups = vec![0u8; width * height];
    for y in 0..height {
        for x in 0..width {
            let index = y * width + x;
            let solid = solid_grid.get(index).copied().unwrap_or(0) != 0;
            let water = liquid_blocks_movement && water_grid.get(index).copied().unwrap_or(0) != 0;
            groups[index] = if solid {
                1
            } else if water {
                2
            } else {
                0
            };
        }
    }

    let mut visited = vec![false; width * height];
    let mut boxes = Vec::new();
    for y in 0..height {
        for x in 0..width {
            let index = y * width + x;
            let group = groups[index];
            if group == 0 || visited[index] {
                continue;
            }

            let mut rect_width = 1;
            while x + rect_width < width {
                let next_index = y * width + x + rect_width;
                if visited[next_index] || groups[next_index] != group {
                    break;
                }
                rect_width += 1;
            }

            let mut rect_height = 1;
            'height_scan: while y + rect_height < height {
                for rx in x..x + rect_width {
                    let next_index = (y + rect_height) * width + rx;
                    if visited[next_index] || groups[next_index] != group {
                        break 'height_scan;
                    }
                }
                rect_height += 1;
            }

            for ry in y..y + rect_height {
                for rx in x..x + rect_width {
                    visited[ry * width + rx] = true;
                }
            }

            boxes.push(CollisionMergeBox {
                x,
                y,
                width: rect_width,
                height: rect_height,
                reason: if group == 1 { "solid" } else { "liquid_policy" },
                source_layer: if group == 1 { LAYER_SOLID } else { LAYER_WATER },
            });
        }
    }
    boxes
}

fn generate_base_terrain_cells(
    coord: LabChunkCoord,
    settings: LabGenerationSettings,
    solid_layer: &[u8],
) -> Vec<LabTerrainCell> {
    let size = settings.effective_chunk_size_cells();
    let mut height_noise = make_fast_noise(
        mix(settings.world_seed, 0x1101),
        settings.terrain_noise_frequency,
    );
    let mut liquid_noise = make_fast_noise(
        mix(settings.world_seed, 0x2202),
        settings.liquid_noise_frequency,
    );
    let mut cells = Vec::with_capacity(size * size);
    for y in 0..size {
        for x in 0..size {
            let world_x = coord.x * size as i32 + x as i32;
            let world_y = coord.z * size as i32 + y as i32;
            let height_percent = noise_percent(&mut height_noise, world_x, world_y);
            let liquid_percent = noise_percent(&mut liquid_noise, world_x + 517, world_y - 911);
            let liquid_threshold = settings.liquid_threshold_percent.clamp(0, 100);
            let liquid =
                height_percent < liquid_threshold || liquid_percent < (liquid_threshold / 2);
            let solid = solid_layer[y * size + x] == 1 && !liquid;
            cells.push(LabTerrainCell {
                height_percent,
                liquid,
                solid,
            });
        }
    }
    cells
}

fn generate_smoothed_solid_layer(coord: LabChunkCoord, settings: LabGenerationSettings) -> Vec<u8> {
    let size = settings.effective_chunk_size_cells();
    let margin = settings.smoothing_passes;
    let expanded_width = size + margin * 2;
    let expanded_height = size + margin * 2;
    let mut solid_noise = make_fast_noise(
        mix(settings.world_seed, 0x3303),
        settings.solid_noise_frequency,
    );
    let mut expanded = vec![0; expanded_width * expanded_height];
    for local_y in 0..expanded_height {
        for local_x in 0..expanded_width {
            let x = local_x as i32 - margin as i32;
            let y = local_y as i32 - margin as i32;
            expanded[local_y * expanded_width + local_x] =
                initial_solid_cell(coord, settings, x, y, &mut solid_noise);
        }
    }
    for _ in 0..margin {
        expanded = smooth_expanded_grid(&expanded, expanded_width, expanded_height);
    }
    let mut grid = vec![0; size * size];
    for y in 0..size {
        for x in 0..size {
            grid[y * size + x] = expanded[(y + margin) * expanded_width + x + margin];
        }
    }
    grid
}

fn initial_solid_cell(
    coord: LabChunkCoord,
    settings: LabGenerationSettings,
    local_x: i32,
    local_y: i32,
    solid_noise: &mut FastNoiseLite,
) -> u8 {
    if settings.wall_threshold_percent <= 0 {
        return 0;
    }
    if settings.wall_threshold_percent >= 100 {
        return 1;
    }
    let size = settings.effective_chunk_size_cells() as i32;
    let world_x = coord.x * size + local_x - 193;
    let world_y = coord.z * size + local_y + 389;
    if noise_percent(solid_noise, world_x, world_y) < settings.wall_threshold_percent.clamp(0, 100)
    {
        1
    } else {
        0
    }
}

fn smooth_expanded_grid(grid: &[u8], width: usize, height: usize) -> Vec<u8> {
    let mut next = grid.to_vec();
    for y in 1..height.saturating_sub(1) {
        for x in 1..width.saturating_sub(1) {
            let mut wall_count = 0;
            for oy in -1..=1 {
                for ox in -1..=1 {
                    if ox == 0 && oy == 0 {
                        continue;
                    }
                    let nx = (x as i32 + ox) as usize;
                    let ny = (y as i32 + oy) as usize;
                    wall_count += i32::from(grid[ny * width + nx]);
                }
            }
            next[y * width + x] = if wall_count >= 5 { 1 } else { 0 };
        }
    }
    next
}

fn carve_rooms_and_paths(
    coord: LabChunkCoord,
    settings: LabGenerationSettings,
    cells: &mut [LabTerrainCell],
    debug_markers: &mut Vec<LabDebugMarker>,
) {
    let size = settings.effective_chunk_size_cells();
    let rooms = room_rects_for_chunk(coord, settings);
    let mut centers = Vec::new();
    for (x, y, width, height) in rooms {
        carve_rect(settings, cells, x, y, width, height);
        let center = (x as i32 + width as i32 / 2, y as i32 + height as i32 / 2);
        centers.push(center);
        debug_markers.push(LabDebugMarker {
            marker_type: "room",
            from: None,
            to: None,
            rect: Some((x as i32, y as i32, width as i32, height as i32)),
        });
    }
    for index in 1..centers.len() {
        carve_path(settings, cells, size, centers[index - 1], centers[index]);
        debug_markers.push(LabDebugMarker {
            marker_type: "path",
            from: Some(centers[index - 1]),
            to: Some(centers[index]),
            rect: None,
        });
    }
}

fn room_rects_for_chunk(
    coord: LabChunkCoord,
    settings: LabGenerationSettings,
) -> Vec<(usize, usize, usize, usize)> {
    let size = settings.effective_chunk_size_cells();
    let clamped_min = settings.room_min_size.clamp(2, size);
    let clamped_max = settings
        .room_max_size
        .max(clamped_min)
        .clamp(clamped_min, size);
    let mut rects = Vec::new();
    for index in 0..settings.room_attempts {
        let mut seed = mix(
            mix(
                mix(settings.world_seed, settings.generator_version),
                coord.x as i64,
            ),
            coord.z as i64,
        );
        seed = mix(seed, index as i64);
        let width = range_from_hash(mix(seed, 11), clamped_min, clamped_max);
        let height = range_from_hash(mix(seed, 17), clamped_min, clamped_max);
        let max_x = size.saturating_sub(width + 1).max(1);
        let max_y = size.saturating_sub(height + 1).max(1);
        let x = range_from_hash(mix(seed, 23), 1, max_x);
        let y = range_from_hash(mix(seed, 29), 1, max_y);
        rects.push((x, y, width, height));
    }
    rects
}

fn carve_rect(
    settings: LabGenerationSettings,
    cells: &mut [LabTerrainCell],
    x: usize,
    y: usize,
    width: usize,
    height: usize,
) {
    let size = settings.effective_chunk_size_cells();
    for cy in y..(y + height).min(size) {
        for cx in x..(x + width).min(size) {
            set_cell_flags(settings, &mut cells[cy * size + cx], false, false);
        }
    }
}

fn carve_path(
    settings: LabGenerationSettings,
    cells: &mut [LabTerrainCell],
    size: usize,
    start: (i32, i32),
    end: (i32, i32),
) {
    let x_step = if end.0 >= start.0 { 1 } else { -1 };
    let mut x = start.0;
    loop {
        if grid_contains(size, x, start.1) {
            set_cell_flags(
                settings,
                &mut cells[start.1 as usize * size + x as usize],
                false,
                false,
            );
        }
        if x == end.0 {
            break;
        }
        x += x_step;
    }
    let y_step = if end.1 >= start.1 { 1 } else { -1 };
    let mut y = start.1;
    loop {
        if grid_contains(size, end.0, y) {
            set_cell_flags(
                settings,
                &mut cells[y as usize * size + end.0 as usize],
                false,
                false,
            );
        }
        if y == end.1 {
            break;
        }
        y += y_step;
    }
}

fn should_enforce_walkable_target(settings: LabGenerationSettings) -> bool {
    settings.wall_threshold_percent > 0
        && settings.wall_threshold_percent < 100
        && !settings.debug_force_chunk_border
}

fn repair_walkable_connectivity(
    settings: LabGenerationSettings,
    cells: &mut [LabTerrainCell],
    debug_markers: &mut Vec<LabDebugMarker>,
) {
    let components = walkable_components(settings, cells);
    if components.len() <= 1 {
        return;
    }
    let anchor = components[0][0];
    for component in components.iter().skip(1) {
        if let Some(start) = component.first().copied() {
            carve_path(
                settings,
                cells,
                settings.effective_chunk_size_cells(),
                start,
                anchor,
            );
            debug_markers.push(LabDebugMarker {
                marker_type: "connectivity_repair",
                from: Some(start),
                to: Some(anchor),
                rect: None,
            });
        }
    }
}

fn balance_walkable_percent(
    settings: LabGenerationSettings,
    cells: &mut [LabTerrainCell],
    debug_markers: &mut Vec<LabDebugMarker>,
) {
    let total = cells.len();
    if total == 0 {
        return;
    }
    let target_min = settings.target_walkable_min_percent.clamp(0, 100);
    let target_max = settings
        .target_walkable_max_percent
        .max(target_min)
        .clamp(target_min, 100);
    let min_count = ((total as f32 * target_min as f32) / 100.0).ceil() as usize;
    let max_count = ((total as f32 * target_max as f32) / 100.0).floor() as usize;
    let mut walkable_count = cells.iter().filter(|cell| cell.walkable(settings)).count();
    if walkable_count < min_count {
        for (x, y) in sorted_cells_by_hash(settings, cells, false) {
            if walkable_count >= min_count {
                break;
            }
            let size = settings.effective_chunk_size_cells();
            set_cell_flags(
                settings,
                &mut cells[y as usize * size + x as usize],
                false,
                false,
            );
            walkable_count += 1;
        }
        debug_markers.push(LabDebugMarker {
            marker_type: "walkability_balance_open",
            from: None,
            to: None,
            rect: None,
        });
    } else if walkable_count > max_count {
        for (x, y) in sorted_cells_by_hash(settings, cells, true) {
            if walkable_count <= max_count {
                break;
            }
            let size = settings.effective_chunk_size_cells();
            set_cell_flags(
                settings,
                &mut cells[y as usize * size + x as usize],
                false,
                true,
            );
            walkable_count -= 1;
        }
        debug_markers.push(LabDebugMarker {
            marker_type: "walkability_balance_block",
            from: None,
            to: None,
            rect: None,
        });
    }
}

fn sorted_cells_by_hash(
    settings: LabGenerationSettings,
    cells: &[LabTerrainCell],
    want_walkable: bool,
) -> Vec<(i32, i32)> {
    let size = settings.effective_chunk_size_cells();
    let mut scored = Vec::new();
    for y in 0..size {
        for x in 0..size {
            let cell = cells[y * size + x];
            if cell.walkable(settings) != want_walkable {
                continue;
            }
            let h = mix(
                mix(
                    mix(settings.world_seed, settings.generator_version),
                    x as i64,
                ),
                y as i64,
            );
            scored.push(((x as i32, y as i32), finalize_hash(h).abs()));
        }
    }
    scored.sort_by_key(|(_, score)| *score);
    scored.into_iter().map(|(cell, _)| cell).collect()
}

fn walkable_components(
    settings: LabGenerationSettings,
    cells: &[LabTerrainCell],
) -> Vec<Vec<(i32, i32)>> {
    let size = settings.effective_chunk_size_cells();
    let mut components = Vec::new();
    let mut visited = vec![false; cells.len()];
    for y in 0..size {
        for x in 0..size {
            let index = y * size + x;
            if visited[index] || !cells[index].walkable(settings) {
                continue;
            }
            let mut component = Vec::new();
            let mut queue = VecDeque::from([(x as i32, y as i32)]);
            visited[index] = true;
            while let Some((cx, cy)) = queue.pop_front() {
                component.push((cx, cy));
                for (nx, ny) in [(cx - 1, cy), (cx + 1, cy), (cx, cy - 1), (cx, cy + 1)] {
                    if !grid_contains(size, nx, ny) {
                        continue;
                    }
                    let neighbor_index = ny as usize * size + nx as usize;
                    if visited[neighbor_index] || !cells[neighbor_index].walkable(settings) {
                        continue;
                    }
                    visited[neighbor_index] = true;
                    queue.push_back((nx, ny));
                }
            }
            components.push(component);
        }
    }
    components.sort_by_key(|component| std::cmp::Reverse(component.len()));
    components
}

fn apply_forced_chunk_border(cells: &mut [LabTerrainCell], size: usize) {
    for i in 0..size {
        set_cell_flags_forced(&mut cells[i], false, true);
        set_cell_flags_forced(&mut cells[(size - 1) * size + i], false, true);
        set_cell_flags_forced(&mut cells[i * size], false, true);
        set_cell_flags_forced(&mut cells[i * size + size - 1], false, true);
    }
}

fn derive_topology_layers(cells: &[LabTerrainCell], size: usize) -> BTreeMap<String, Vec<u8>> {
    let mut ground = vec![0; size * size];
    let mut solid = vec![0; size * size];
    let mut water = vec![0; size * size];
    let cliff = vec![0; size * size];
    for (index, cell) in cells.iter().enumerate() {
        ground[index] = if cell.liquid { 0 } else { 1 };
        solid[index] = if cell.solid { 1 } else { 0 };
        water[index] = if cell.liquid { 1 } else { 0 };
    }
    BTreeMap::from([
        (LAYER_CLIFF.to_string(), cliff),
        (LAYER_GROUND.to_string(), ground),
        (LAYER_SOLID.to_string(), solid),
        (LAYER_WATER.to_string(), water),
    ])
}

fn set_cell_flags(
    settings: LabGenerationSettings,
    cell: &mut LabTerrainCell,
    liquid: bool,
    solid: bool,
) {
    cell.liquid = liquid;
    cell.solid = solid;
    if !cell.walkable(settings) && !solid && !liquid {
        cell.height_percent = cell.height_percent.clamp(0, 100);
    }
}

fn set_cell_flags_forced(cell: &mut LabTerrainCell, liquid: bool, solid: bool) {
    cell.liquid = liquid;
    cell.solid = solid;
}

fn make_fast_noise(seed: i64, frequency: f32) -> FastNoiseLite {
    let mut noise = FastNoiseLite::new();
    noise.set_seed(Some(seed as i32));
    noise.set_noise_type(Some(NoiseType::OpenSimplex2));
    noise.set_fractal_type(Some(FractalType::FBm));
    noise.set_fractal_octaves(Some(1));
    noise.set_frequency(Some(frequency.max(0.001)));
    noise
}

fn noise_percent(noise: &mut FastNoiseLite, x: i32, y: i32) -> i32 {
    let sample = noise.get_noise_2d(x as f32, y as f32);
    (((sample * 0.5 + 0.5) * 100.0).round() as i32).clamp(0, 100)
}

fn range_from_hash(hash_value: i64, min_value: usize, max_value: usize) -> usize {
    if max_value <= min_value {
        return min_value;
    }
    min_value + (finalize_hash(hash_value).abs() as usize % (max_value - min_value + 1))
}

fn grid_contains(size: usize, x: i32, y: i32) -> bool {
    x >= 0 && y >= 0 && (x as usize) < size && (y as usize) < size
}

fn grid_value(grid: &[u8], size: usize, x: usize, y: usize) -> u8 {
    grid.get(y * size + x).copied().unwrap_or(0)
}

fn topology_layer_grid_value(
    topology_layers: &BTreeMap<String, Vec<u8>>,
    layer_id: &str,
    size: usize,
    x: usize,
    y: usize,
) -> u8 {
    topology_layers
        .get(layer_id)
        .or_else(|| topology_layers.get(LAYER_SOLID))
        .map(|grid| grid_value(grid, size, x, y))
        .unwrap_or(0)
}

fn floor_div(value: i32, divisor: i32) -> i32 {
    if divisor <= 0 {
        return 0;
    }
    (value as f32 / divisor as f32).floor() as i32
}

fn positive_mod(value: i32, modulus: i32) -> i32 {
    if modulus <= 0 {
        return 0;
    }
    let result = value % modulus;
    if result < 0 { result + modulus } else { result }
}

fn mix(seed: i64, value: i64) -> i64 {
    let mut h = seed ^ value.wrapping_mul(0x45d9f3b);
    h ^= h >> 16;
    h = h.wrapping_mul(0x45d9f3b);
    h ^= h >> 16;
    h
}

fn finalize_hash(value: i64) -> i64 {
    let mut h = value;
    h ^= h >> 16;
    h = h.wrapping_mul(0x7feb352d);
    h ^= h >> 15;
    h = h.wrapping_mul(0x846ca68b);
    h ^= h >> 16;
    h
}

#[cfg(test)]
mod tests {
    use super::*;
    use tile_topology::Occupancy;

    fn snapshot(grid: &DenseGrid2D<CellType>) -> Vec<CellType> {
        grid.cells().to_vec()
    }

    #[test]
    fn seeded_generator_is_deterministic() {
        let mut a = SeededGridGenerator::new(42);
        let mut b = SeededGridGenerator::new(42);

        assert_eq!(a.next_seed(), b.next_seed());
        assert_eq!(
            a.noise_fill_settings(0.1, 0.8),
            b.noise_fill_settings(0.1, 0.8)
        );
    }

    #[test]
    fn noise_fill_is_deterministic_for_same_seed() {
        let settings = NoiseFillSettings::new(0.1, 0.65, 123);
        let mut a = DenseGrid2D::new(8, 8, CellType::Empty);
        let mut b = DenseGrid2D::new(8, 8, CellType::Empty);

        fill_with_noise(&mut a, settings);
        fill_with_noise(&mut b, settings);

        assert_eq!(snapshot(&a), snapshot(&b));
    }

    #[test]
    fn test_pattern_creates_non_empty_grid() {
        let grid = test_pattern(5, 5);

        assert!(grid.get(GridCoord2::new(0, 0)).unwrap().is_occupied());
        assert!(grid.get(GridCoord2::new(2, 2)).is_some());
    }

    #[test]
    fn lab_generation_is_deterministic_for_same_settings_and_coord() {
        let settings = LabGenerationSettings::default();
        let coord = LabChunkCoord::new(2, 0, -3);

        assert_eq!(
            generate_lab_chunk(coord, settings),
            generate_lab_chunk(coord, settings)
        );
    }

    #[test]
    fn owned_halo_formation_has_expected_size_and_sources() {
        let settings = LabGenerationSettings::default();
        let coord = LabChunkCoord::new(0, 0, 0);
        let chunk = generate_lab_chunk(coord, settings);
        let solid = chunk.topology_layers.get(LAYER_SOLID).unwrap();
        let formation = build_owned_halo_formation_layer(coord, settings, LAYER_SOLID, solid);

        assert_eq!(formation.size, settings.effective_chunk_size_cells() + 1);
        assert_eq!(
            formation.formation_grid.len(),
            formation.size * formation.size
        );
        assert!(formation.source_chunk_coords.contains(&coord));
        assert!(formation.source_chunk_coords.len() >= 4);
    }

    #[test]
    fn batched_owned_halo_matches_single_layer_output() {
        let settings = LabGenerationSettings::default();
        let coord = LabChunkCoord::new(0, 0, 0);
        let chunk = generate_lab_chunk(coord, settings);
        let requested = vec![LAYER_SOLID.to_string(), LAYER_WATER.to_string()];
        let batch =
            build_owned_halo_formation_layers(coord, settings, &chunk.topology_layers, &requested);

        for layer_id in requested {
            let single = build_owned_halo_formation_layer(
                coord,
                settings,
                layer_id.as_str(),
                chunk.topology_layers.get(layer_id.as_str()).unwrap(),
            );
            assert_eq!(batch.layers.get(layer_id.as_str()), Some(&single));
        }
    }

    #[test]
    fn batched_owned_halo_generates_each_neighbor_once() {
        let settings = LabGenerationSettings::default();
        let coord = LabChunkCoord::new(0, 0, 0);
        let chunk = generate_lab_chunk(coord, settings);
        let requested = vec![
            LAYER_GROUND.to_string(),
            LAYER_WATER.to_string(),
            LAYER_SOLID.to_string(),
            LAYER_CLIFF.to_string(),
        ];
        let batch =
            build_owned_halo_formation_layers(coord, settings, &chunk.topology_layers, &requested);
        let neighbor_source_count = batch
            .source_chunk_coords
            .iter()
            .filter(|source_coord| **source_coord != coord)
            .count();

        assert_eq!(batch.layers.len(), requested.len());
        assert_eq!(batch.neighbor_generation_count, neighbor_source_count);
        assert!(batch.neighbor_generation_count <= 3);
    }

    #[test]
    fn collision_merge_covers_full_grid_as_one_box() {
        let solid = vec![1; 16];
        let boxes = merge_collision_cells(4, 4, &solid, &[], true);

        assert_eq!(boxes.len(), 1);
        assert_eq!(boxes[0].cell_count(), 16);
        assert_eq!(boxes[0].source_layer, LAYER_SOLID);
    }

    #[test]
    fn collision_merge_keeps_checkerboard_unmerged() {
        let solid = vec![1, 0, 1, 0, 0, 1, 0, 1, 1, 0, 1, 0, 0, 1, 0, 1];
        let boxes = merge_collision_cells(4, 4, &solid, &[], true);
        let covered: usize = boxes.iter().map(CollisionMergeBox::cell_count).sum();

        assert_eq!(boxes.len(), 8);
        assert_eq!(covered, 8);
    }

    #[test]
    fn collision_merge_keeps_solid_and_liquid_separate() {
        let solid = vec![1, 1, 0, 0, 1, 1, 0, 0];
        let water = vec![0, 0, 1, 1, 0, 0, 1, 1];
        let boxes = merge_collision_cells(4, 2, &solid, &water, true);

        assert_eq!(boxes.len(), 2);
        assert!(
            boxes
                .iter()
                .any(|b| b.source_layer == LAYER_SOLID && b.cell_count() == 4)
        );
        assert!(
            boxes
                .iter()
                .any(|b| b.source_layer == LAYER_WATER && b.cell_count() == 4)
        );
    }

    #[test]
    fn collision_merge_ignores_liquid_when_policy_allows_open_water() {
        let solid = vec![0, 0, 0, 0];
        let water = vec![1, 1, 1, 1];
        let boxes = merge_collision_cells(2, 2, &solid, &water, false);

        assert!(boxes.is_empty());
    }
}
