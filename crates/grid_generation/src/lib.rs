pub mod bounded;

use fastnoise_lite::{FastNoiseLite, FractalType, NoiseType};
use grid_math::{ALL_NEIGHBORS_2D, GridCoord2, GridSize2};
use grid_storage::DenseGrid2D;
use rand::{Rng, SeedableRng, rngs::StdRng};
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
}
