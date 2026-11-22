//! Grid #2 research: characterize how a finite sampled scalar Noise2D and an
//! explicit threshold could preserve the existing fused fill_with_noise API.
//!
//! All candidates here are TEST-LOCAL. No production Grid public API, node,
//! Riesel capability, renderer or generic scheduler is introduced.
#![forbid(unsafe_code)]

use fastnoise_lite::{FastNoiseLite, FractalType, NoiseType};
use grid_generation::{NoiseFillSettings, cellular_automata_step, fill_with_noise};
use grid_math::{ALL_NEIGHBORS_2D, GridCoord2, GridSize2};
use grid_storage::DenseGrid2D;
use tile_topology::{CellType, form_visual_tiles};

const FIELD_BYTES: usize = std::mem::size_of::<f32>();

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CandidateError {
    EmptySize,
    CoordNotRepresentable,
    CellCountOverflow,
    ExceedsBudget,
    InvalidScale,
    InvalidThreshold,
}

fn check_candidate_request(
    size: GridSize2,
    scale: f32,
    threshold: f32,
    max_field_bytes: usize,
) -> Result<usize, CandidateError> {
    if size.is_empty() {
        return Err(CandidateError::EmptySize);
    }
    if !size.fits_i32_coords() {
        return Err(CandidateError::CoordNotRepresentable);
    }
    let count = size
        .checked_area()
        .ok_or(CandidateError::CellCountOverflow)?;
    let bytes = count
        .checked_mul(FIELD_BYTES)
        .ok_or(CandidateError::CellCountOverflow)?;
    if bytes > max_field_bytes {
        return Err(CandidateError::ExceedsBudget);
    }
    if !scale.is_finite() || scale <= 0.0 {
        return Err(CandidateError::InvalidScale);
    }
    if !threshold.is_finite() || !(0.0..=1.0).contains(&threshold) {
        return Err(CandidateError::InvalidThreshold);
    }
    Ok(count)
}

/// Candidate finite eager sample (not a published API).
/// Preserve the exact existing FastNoiseLite version and operation order,
/// including domain warp and remap; changing arithmetic or representation
/// here can change a threshold decision.
fn candidate_sample_legacy_noise(size: GridSize2, settings: NoiseFillSettings) -> DenseGrid2D<f32> {
    let mut noise = FastNoiseLite::new();
    noise.set_seed(Some(i32::from_ne_bytes(settings.seed.to_ne_bytes())));
    noise.set_noise_type(Some(NoiseType::OpenSimplex2));
    noise.set_fractal_type(Some(FractalType::FBm));
    noise.set_fractal_octaves(Some(4));
    noise.set_frequency(Some(settings.scale));

    DenseGrid2D::from_fn(size, |coord| {
        let (warped_x, warped_y) = noise.domain_warp_2d(coord.x as f32, coord.y as f32);
        let value = (noise.get_noise_2d(warped_x, warped_y) + 1.0) / 2.0;
        (value * 1.3).clamp(0.0, 1.0)
    })
}

fn candidate_threshold(field: &DenseGrid2D<f32>, threshold: f32) -> DenseGrid2D<CellType> {
    DenseGrid2D::from_fn(field.size(), |coord| {
        if *field.get(coord).expect("sample coordinate") > threshold {
            CellType::Wall
        } else {
            CellType::Empty
        }
    })
}

#[derive(Debug, Copy, Clone)]
enum Outside {
    Empty,
    Occupied,
}

#[derive(Debug, Copy, Clone)]
struct CandidateCa {
    birth_min: usize,
    survive_min: usize,
    outside: Outside,
    protected_border: usize,
}

fn is_border(coord: GridCoord2, size: GridSize2, thickness: usize) -> bool {
    let x = usize::try_from(coord.x).expect("in-bounds x");
    let y = usize::try_from(coord.y).expect("in-bounds y");
    x < thickness
        || y < thickness
        || x >= size.width.saturating_sub(thickness)
        || y >= size.height.saturating_sub(thickness)
}

fn candidate_ca_step(input: &DenseGrid2D<CellType>, rule: CandidateCa) -> DenseGrid2D<CellType> {
    DenseGrid2D::from_fn(input.size(), |coord| {
        let previous = *input.get(coord).expect("in-bounds cell");
        if is_border(coord, input.size(), rule.protected_border) {
            return previous;
        }
        let neighbors = ALL_NEIGHBORS_2D
            .iter()
            .filter(|offset| match input.get(coord + **offset) {
                Some(CellType::Wall) => true,
                Some(CellType::Empty) => false,
                None => matches!(rule.outside, Outside::Occupied),
            })
            .count();

        let min = match previous {
            CellType::Empty => rule.birth_min,
            CellType::Wall => rule.survive_min,
        };
        if neighbors >= min {
            CellType::Wall
        } else {
            CellType::Empty
        }
    })
}

#[test]
fn separate_scalar_sampling_and_threshold_match_existing_fused_noise_at_every_cell() {
    let size = GridSize2::new(64, 64);
    for seed in [0u32, 1, 42, u32::MAX] {
        for scale in [0.01f32, 0.10, 0.29] {
            for threshold in [0.0f32, 0.32, 0.55, 1.0] {
                let settings = NoiseFillSettings::new(scale, threshold, seed);
                let mut legacy = DenseGrid2D::new(64, 64, CellType::Empty);
                fill_with_noise(&mut legacy, settings);

                let scalar = candidate_sample_legacy_noise(size, settings);
                let separate = candidate_threshold(&scalar, threshold);

                assert_eq!(
                    legacy, separate,
                    "seed={seed} scale={scale} threshold={threshold}"
                );
                assert!(
                    scalar
                        .cells()
                        .iter()
                        .all(|x| x.is_finite() && (0.0..=1.0).contains(x))
                );
            }
        }
    }
}

#[test]
fn sampled_scalar_is_reusable_as_a_non_room_data_product() {
    let settings = NoiseFillSettings::new(0.07, 0.6, 42);
    let a = candidate_sample_legacy_noise(GridSize2::new(64, 64), settings);
    let b = candidate_sample_legacy_noise(GridSize2::new(64, 64), settings);
    assert_eq!(a.cells(), b.cells());
    let stats = a
        .cells()
        .iter()
        .fold((f32::INFINITY, f32::NEG_INFINITY, 0.0f64), |acc, v| {
            (acc.0.min(*v), acc.1.max(*v), acc.2 + f64::from(*v))
        });
    let mean = stats.2 / a.len() as f64;
    assert!((0.0..=1.0).contains(&mean));
    assert!(stats.0 <= stats.1);
    assert_eq!(a.size(), GridSize2::new(64, 64));
    // Same *scalar* buffer supports an independent mean/heat-statistics
    // consumer, without a room/dungeon occupancy representation.
}

#[test]
fn strict_threshold_boundary_is_explicit_and_not_an_implicit_port_cast() {
    let field = DenseGrid2D::from_fn(GridSize2::new(3, 1), |coord| {
        [0.0f32, 0.5, 1.0][coord.x as usize]
    });
    assert_eq!(
        candidate_threshold(&field, 0.5).cells(),
        &[CellType::Empty, CellType::Empty, CellType::Wall]
    );
    assert_eq!(
        candidate_threshold(&field, 1.0).cells(),
        &[CellType::Empty, CellType::Empty, CellType::Empty]
    );
}

#[test]
fn proposed_legacy_ca_profile_reproduces_existing_fixed_rule_exactly() {
    let rule = CandidateCa {
        birth_min: 5,
        survive_min: 5,
        outside: Outside::Occupied,
        protected_border: 0,
    };
    for (width, height) in [(1, 1), (1, 9), (9, 1), (2, 2), (16, 17), (64, 64)] {
        let size = GridSize2::new(width, height);
        let source = DenseGrid2D::from_fn(size, |coord| {
            if (coord.x * 13 + coord.y * 7) % 11 < 5 {
                CellType::Wall
            } else {
                CellType::Empty
            }
        });
        assert_eq!(
            candidate_ca_step(&source, rule),
            cellular_automata_step(&source)
        );
    }
}

#[test]
fn protected_mixed_border_remains_bit_exact_through_each_ca_step() {
    let size = GridSize2::new(64, 64);
    let noise = candidate_sample_legacy_noise(size, NoiseFillSettings::new(0.09, 0.55, 42));
    let mut input = candidate_threshold(&noise, 0.55);
    for (coord, cell) in input.iter_cells_mut() {
        if is_border(coord, size, 1) {
            *cell = if (coord.x + coord.y) % 2 == 0 {
                CellType::Wall
            } else {
                CellType::Empty
            };
        }
    }
    let original = input.clone();
    let rule = CandidateCa {
        birth_min: 5,
        survive_min: 4,
        protected_border: 1,
        outside: Outside::Occupied,
    };
    let mut current = input;
    for iteration in 1..=5 {
        current = candidate_ca_step(&current, rule);
        for (coord, cell) in current.iter_cells() {
            if is_border(coord, size, 1) {
                assert_eq!(
                    Some(cell),
                    original.get(coord),
                    "step {iteration} coordinate {coord:?}"
                );
            }
        }
    }
    let visual = form_visual_tiles(&current);
    assert_eq!((visual.width(), visual.height()), (65, 65));
}

#[test]
fn boundary_reads_and_write_protection_are_independent_rules() {
    let input = DenseGrid2D::new(1, 1, CellType::Empty);
    let base = CandidateCa {
        birth_min: 8,
        survive_min: 8,
        protected_border: 0,
        outside: Outside::Empty,
    };
    assert_eq!(candidate_ca_step(&input, base).cells(), &[CellType::Empty]);
    assert_eq!(
        candidate_ca_step(
            &input,
            CandidateCa {
                outside: Outside::Occupied,
                ..base
            }
        )
        .cells(),
        &[CellType::Wall]
    );
    assert_eq!(
        candidate_ca_step(
            &input,
            CandidateCa {
                outside: Outside::Occupied,
                protected_border: 1,
                ..base
            }
        )
        .cells(),
        &[CellType::Empty]
    );
}

#[test]
fn a_fallible_node_request_gate_precedes_infallible_dense_grid_constructors() {
    assert_eq!(
        check_candidate_request(GridSize2::new(64, 64), 0.1, 0.5, 64 * 64 * FIELD_BYTES),
        Ok(4096),
    );
    assert_eq!(
        check_candidate_request(GridSize2::new(0, 64), 0.1, 0.5, usize::MAX),
        Err(CandidateError::EmptySize),
    );
    assert_eq!(
        check_candidate_request(
            GridSize2::new(i32::MAX as usize + 1, 1),
            0.1,
            0.5,
            usize::MAX
        ),
        Err(CandidateError::CoordNotRepresentable),
    );
    assert_eq!(
        check_candidate_request(GridSize2::new(1024, 1024), 0.1, 0.5, 8192),
        Err(CandidateError::ExceedsBudget),
    );
    assert_eq!(
        check_candidate_request(GridSize2::new(2, 2), f32::NAN, 0.5, 4096),
        Err(CandidateError::InvalidScale),
    );
    assert_eq!(
        check_candidate_request(GridSize2::new(2, 2), 0.1, f32::INFINITY, 4096),
        Err(CandidateError::InvalidThreshold),
    );
    assert_eq!(
        check_candidate_request(GridSize2::new(2, 2), -0.1, 0.5, 4096),
        Err(CandidateError::InvalidScale),
    );
    assert_eq!(
        check_candidate_request(GridSize2::new(2, 2), 0.1, -0.1, 4096),
        Err(CandidateError::InvalidThreshold),
    );
    // The choice to reject empty/negative/out-of-range requests is a
    // RESEARCH PROPOSAL, not an accepted change to existing legacy APIs.
}
