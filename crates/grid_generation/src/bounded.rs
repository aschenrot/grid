//! Bounded, engine-neutral finite-grid transforms.
//!
//! These functions return local zero-origin dense grids. The caller must keep
//! sampling origin, seed and algorithm identity with any persisted artifact.

use fastnoise_lite::{FastNoiseLite, FractalType, NoiseType};
use grid_math::{ALL_NEIGHBORS_2D, GridCoord2, GridSize2, ORTHOGONAL_NEIGHBORS_2D};
use grid_storage::{DenseGrid2D, DenseGridBuildError, GridRead};
use tile_topology::Occupancy;

/// Failures that are checked before allocating or executing a transform.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum GenerationError {
    EmptySize,
    CoordinateDimensionsOutOfRange,
    RegionOutOfRange,
    CellCountOverflow,
    ExceedsOutputBudget,
    AllocationFailed,
    InvalidFrequency,
    InvalidThreshold,
    NonFiniteSample,
    InvalidRule,
    ProtectionSizeMismatch,
}

impl From<DenseGridBuildError> for GenerationError {
    fn from(value: DenseGridBuildError) -> Self {
        match value {
            DenseGridBuildError::CoordinateDimensionsOutOfRange => {
                Self::CoordinateDimensionsOutOfRange
            }
            DenseGridBuildError::CellCountOverflow => Self::CellCountOverflow,
            DenseGridBuildError::AllocationFailed => Self::AllocationFailed,
            DenseGridBuildError::CellCountMismatch { .. } => Self::CellCountOverflow,
        }
    }
}

fn checked_output<T>(size: GridSize2, max_output_bytes: usize) -> Result<(), GenerationError> {
    if size.is_empty() {
        return Err(GenerationError::EmptySize);
    }
    if !size.fits_i32_coords() {
        return Err(GenerationError::CoordinateDimensionsOutOfRange);
    }
    let count = size
        .checked_area()
        .ok_or(GenerationError::CellCountOverflow)?;
    let bytes = count
        .checked_mul(std::mem::size_of::<T>())
        .ok_or(GenerationError::CellCountOverflow)?;
    if bytes > max_output_bytes {
        return Err(GenerationError::ExceedsOutputBudget);
    }
    Ok(())
}

/// Pinned sampling profile: legacy warped OpenSimplex2, four FBm octaves,
/// using the existing FastNoiseLite implementation and remapping order.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum NoiseAlgorithm2D {
    LegacyWarpedOpenSimplex2Fbm4V1,
}

/// Origin names the corresponding world-space sample location, while the
/// returned dense grid remains zero-origin local storage.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct NoiseSample2D {
    pub seed: u32,
    pub frequency: f32,
    pub origin: GridCoord2,
    pub algorithm: NoiseAlgorithm2D,
}

impl NoiseSample2D {
    pub const fn legacy(seed: u32, frequency: f32, origin: GridCoord2) -> Self {
        Self {
            seed,
            frequency,
            origin,
            algorithm: NoiseAlgorithm2D::LegacyWarpedOpenSimplex2Fbm4V1,
        }
    }
}

/// Samples a finite scalar field, allocating at most the requested output
/// byte budget (excluding allocator metadata). The caller owns snapshots.
pub fn sample_noise_2d(
    size: GridSize2,
    settings: NoiseSample2D,
    max_output_bytes: usize,
) -> Result<DenseGrid2D<f32>, GenerationError> {
    checked_output::<f32>(size, max_output_bytes)?;
    if !settings.frequency.is_finite() || settings.frequency <= 0.0 {
        return Err(GenerationError::InvalidFrequency);
    }

    for (origin, extent) in [
        (settings.origin.x, size.width),
        (settings.origin.y, size.height),
    ] {
        let last = i64::from(origin) + (extent - 1) as i64;
        if i32::try_from(last).is_err() {
            return Err(GenerationError::RegionOutOfRange);
        }
    }

    let mut noise = FastNoiseLite::new();
    noise.set_seed(Some(i32::from_ne_bytes(settings.seed.to_ne_bytes())));
    match settings.algorithm {
        NoiseAlgorithm2D::LegacyWarpedOpenSimplex2Fbm4V1 => {
            noise.set_noise_type(Some(NoiseType::OpenSimplex2));
            noise.set_fractal_type(Some(FractalType::FBm));
            noise.set_fractal_octaves(Some(4));
            noise.set_frequency(Some(settings.frequency));
        }
    }

    let samples = DenseGrid2D::try_from_fn(size, |local| {
        let x = (i64::from(settings.origin.x) + i64::from(local.x)) as f32;
        let y = (i64::from(settings.origin.y) + i64::from(local.y)) as f32;
        let (warped_x, warped_y) = noise.domain_warp_2d(x, y);
        let value = (noise.get_noise_2d(warped_x, warped_y) + 1.0) / 2.0;
        (value * 1.3).clamp(0.0, 1.0)
    })
    .map_err(GenerationError::from)?;
    if samples.cells().iter().any(|value| !value.is_finite()) {
        return Err(GenerationError::NonFiniteSample);
    }
    Ok(samples)
}

/// Explicit scalar-to-occupancy comparison; never a hidden port coercion.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ThresholdComparison {
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,
}

/// Creates an occupancy grid. Non-finite samples and threshold values fail
/// before allocation instead of silently mapping NaN to empty.
pub fn threshold_2d(
    samples: &DenseGrid2D<f32>,
    threshold: f32,
    comparison: ThresholdComparison,
    max_output_bytes: usize,
) -> Result<DenseGrid2D<bool>, GenerationError> {
    checked_output::<bool>(samples.size(), max_output_bytes)?;
    if !threshold.is_finite() {
        return Err(GenerationError::InvalidThreshold);
    }
    if samples.cells().iter().any(|sample| !sample.is_finite()) {
        return Err(GenerationError::NonFiniteSample);
    }
    DenseGrid2D::try_from_fn(samples.size(), |coord| {
        let value = *samples.get(coord).expect("validated dense coordinate");
        match comparison {
            ThresholdComparison::GreaterThan => value > threshold,
            ThresholdComparison::GreaterThanOrEqual => value >= threshold,
            ThresholdComparison::LessThan => value < threshold,
            ThresholdComparison::LessThanOrEqual => value <= threshold,
        }
    })
    .map_err(Into::into)
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum CaNeighborhood2D {
    Moore8,
    VonNeumann4,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum CaOutsideRead {
    Empty,
    Occupied,
}

/// An immutable synchronous CA rule. Protected cells retain their previous
/// values but still contribute to neighbor reads as normal.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct CaRules2D {
    pub neighborhood: CaNeighborhood2D,
    pub birth_at_least: u8,
    pub survive_at_least: u8,
    pub outside_read: CaOutsideRead,
    pub protected_border: usize,
}

impl CaRules2D {
    /// Matches the fixed legacy CA rule for the same occupancy input.
    pub const fn legacy() -> Self {
        Self {
            neighborhood: CaNeighborhood2D::Moore8,
            birth_at_least: 5,
            survive_at_least: 5,
            outside_read: CaOutsideRead::Occupied,
            protected_border: 0,
        }
    }
}

fn is_protected_border(coord: GridCoord2, size: GridSize2, thickness: usize) -> bool {
    let x = coord.x as usize;
    let y = coord.y as usize;
    x < thickness
        || y < thickness
        || x >= size.width.saturating_sub(thickness)
        || y >= size.height.saturating_sub(thickness)
}

/// Performs exactly one synchronous step over the previous immutable input.
/// Optional protected mask is a same-size local dense boolean grid; true
/// cells cannot be mutated, independently of the out-of-range read policy.
pub fn cellular_automata_step_2d<G>(
    input: &G,
    rules: CaRules2D,
    protected: Option<&DenseGrid2D<bool>>,
    max_output_bytes: usize,
) -> Result<DenseGrid2D<bool>, GenerationError>
where
    G: GridRead,
    G::Cell: Occupancy,
{
    let size = input.size();
    checked_output::<bool>(size, max_output_bytes)?;
    if protected.is_some_and(|mask| mask.size() != size) {
        return Err(GenerationError::ProtectionSizeMismatch);
    }

    let (offsets, max_neighbors): (&[GridCoord2], u8) = match rules.neighborhood {
        CaNeighborhood2D::Moore8 => (&ALL_NEIGHBORS_2D, 8),
        CaNeighborhood2D::VonNeumann4 => (&ORTHOGONAL_NEIGHBORS_2D, 4),
    };
    if rules.birth_at_least > max_neighbors + 1 || rules.survive_at_least > max_neighbors + 1 {
        return Err(GenerationError::InvalidRule);
    }

    DenseGrid2D::try_from_fn(size, |coord| {
        let previous = input
            .get(coord)
            .expect("validated local input coordinate")
            .is_occupied();
        if is_protected_border(coord, size, rules.protected_border)
            || protected.is_some_and(|mask| mask.get(coord).copied().unwrap_or(false))
        {
            return previous;
        }

        let occupied = offsets
            .iter()
            .filter(|offset| match input.get(coord + **offset) {
                Some(cell) => cell.is_occupied(),
                None => matches!(rules.outside_read, CaOutsideRead::Occupied),
            })
            .count();
        let min = if previous {
            rules.survive_at_least
        } else {
            rules.birth_at_least
        };
        occupied >= usize::from(min)
    })
    .map_err(Into::into)
}
