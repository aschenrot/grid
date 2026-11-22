use grid_generation::bounded::{
    CaNeighborhood2D, CaOutsideRead, CaRules2D, GenerationError, NoiseSample2D,
    ThresholdComparison, cellular_automata_step_2d, sample_noise_2d, threshold_2d,
};
use grid_generation::{NoiseFillSettings, cellular_automata_step, fill_with_noise};
use grid_math::{GridCoord2, GridSize2};
use grid_storage::DenseGrid2D;
use tile_topology::{CellType, form_visual_tiles};

const MAX_BYTES: usize = 1024 * 1024;

#[test]
fn legacy_fused_noise_matches_explicit_sample_and_threshold_at_every_cell() {
    for seed in [0, 1, 42, u32::MAX] {
        for frequency in [0.01, 0.1, 0.29] {
            let scalar = sample_noise_2d(
                GridSize2::new(64, 64),
                NoiseSample2D::legacy(seed, frequency, GridCoord2::ZERO),
                MAX_BYTES,
            )
            .unwrap();
            assert!(scalar.cells().iter().all(|value| value.is_finite()));
            for threshold in [0.0, 0.32, 0.55, 1.0] {
                let output = threshold_2d(
                    &scalar,
                    threshold,
                    ThresholdComparison::GreaterThan,
                    MAX_BYTES,
                )
                .unwrap();
                let mut legacy = DenseGrid2D::new(64, 64, CellType::Empty);
                fill_with_noise(
                    &mut legacy,
                    NoiseFillSettings::new(frequency, threshold, seed),
                );
                assert_eq!(
                    output.cells().iter().copied().collect::<Vec<_>>(),
                    legacy
                        .cells()
                        .iter()
                        .map(|cell| *cell == CellType::Wall)
                        .collect::<Vec<_>>(),
                    "seed={seed} frequency={frequency} threshold={threshold}"
                );
            }
        }
    }
}

#[test]
fn translated_sampling_matches_legacy_origin_at_corresponding_coordinates() {
    let full = sample_noise_2d(
        GridSize2::new(12, 10),
        NoiseSample2D::legacy(123, 0.13, GridCoord2::ZERO),
        MAX_BYTES,
    )
    .unwrap();
    let region = sample_noise_2d(
        GridSize2::new(4, 3),
        NoiseSample2D::legacy(123, 0.13, GridCoord2::new(5, 6)),
        MAX_BYTES,
    )
    .unwrap();
    for (local, value) in region.iter_cells() {
        assert_eq!(
            Some(value),
            full.get(GridCoord2::new(local.x + 5, local.y + 6))
        );
    }
}

#[test]
fn scalar_field_supports_independent_numeric_processing() {
    let a = sample_noise_2d(
        GridSize2::new(64, 64),
        NoiseSample2D::legacy(7, 0.11, GridCoord2::ZERO),
        MAX_BYTES,
    )
    .unwrap();
    let b = sample_noise_2d(
        a.size(),
        NoiseSample2D::legacy(7, 0.11, GridCoord2::ZERO),
        MAX_BYTES,
    )
    .unwrap();
    assert_eq!(a, b);
    let average = a.cells().iter().map(|x| f64::from(*x)).sum::<f64>() / a.len() as f64;
    assert!((0.0..=1.0).contains(&average));
}

#[test]
fn threshold_is_explicit_and_rejects_nonfinite_values() {
    let field = DenseGrid2D::from_fn(GridSize2::new(3, 1), |coord| {
        [0.0, 0.5, 1.0][coord.x as usize]
    });
    assert_eq!(
        threshold_2d(&field, 0.5, ThresholdComparison::GreaterThan, MAX_BYTES)
            .unwrap()
            .cells(),
        &[false, false, true]
    );
    assert_eq!(
        threshold_2d(
            &field,
            0.5,
            ThresholdComparison::GreaterThanOrEqual,
            MAX_BYTES
        )
        .unwrap()
        .cells(),
        &[false, true, true]
    );
    let nan = DenseGrid2D::new(1, 1, f32::NAN);
    assert_eq!(
        threshold_2d(&nan, 0.5, ThresholdComparison::GreaterThan, MAX_BYTES),
        Err(GenerationError::NonFiniteSample)
    );
}

#[test]
fn new_ca_legacy_profile_matches_existing_fixed_rule_on_thin_and_regular_grids() {
    for (width, height) in [(1, 1), (1, 9), (9, 1), (2, 2), (16, 17), (64, 64)] {
        let input = DenseGrid2D::from_fn(GridSize2::new(width, height), |coord| {
            if (coord.x * 13 + coord.y * 7) % 11 < 5 {
                CellType::Wall
            } else {
                CellType::Empty
            }
        });
        let actual =
            cellular_automata_step_2d(&input, CaRules2D::legacy(), None, MAX_BYTES).unwrap();
        let expected = cellular_automata_step(&input);
        for (a, b) in actual.cells().iter().zip(expected.cells()) {
            assert_eq!(*a, *b == CellType::Wall);
        }
    }
}

#[test]
fn protected_border_and_arbitrary_mask_remain_frozen_on_every_iteration() {
    let size = GridSize2::new(64, 64);
    let input = DenseGrid2D::from_fn(size, |c| (c.x + 3 * c.y) % 7 < 3);
    let protected = DenseGrid2D::from_fn(size, |c| c.x == 14 || (c.x == 30 && c.y == 20));
    let rules = CaRules2D {
        birth_at_least: 5,
        survive_at_least: 4,
        protected_border: 1,
        neighborhood: CaNeighborhood2D::Moore8,
        outside_read: CaOutsideRead::Occupied,
    };
    let mut current = input.clone();
    for iteration in 0..5 {
        current = cellular_automata_step_2d(&current, rules, Some(&protected), MAX_BYTES).unwrap();
        for (coord, value) in current.iter_cells() {
            let border = coord.x == 0 || coord.y == 0 || coord.x == 63 || coord.y == 63;
            if border || protected.get(coord) == Some(&true) {
                assert_eq!(
                    Some(value),
                    input.get(coord),
                    "iteration {iteration} at {coord:?}"
                );
            }
        }
    }
    let visual = form_visual_tiles(&current);
    assert_eq!(visual.size(), GridSize2::new(65, 65));
}

#[test]
fn outside_reads_do_not_change_write_protection() {
    let input = DenseGrid2D::new(1, 1, false);
    let rules = CaRules2D {
        birth_at_least: 8,
        survive_at_least: 8,
        neighborhood: CaNeighborhood2D::Moore8,
        outside_read: CaOutsideRead::Empty,
        protected_border: 0,
    };
    assert_eq!(
        cellular_automata_step_2d(&input, rules, None, MAX_BYTES)
            .unwrap()
            .cells(),
        &[false]
    );
    let occupied = CaRules2D {
        outside_read: CaOutsideRead::Occupied,
        ..rules
    };
    assert_eq!(
        cellular_automata_step_2d(&input, occupied, None, MAX_BYTES)
            .unwrap()
            .cells(),
        &[true]
    );
    assert_eq!(
        cellular_automata_step_2d(
            &input,
            CaRules2D {
                protected_border: 1,
                ..occupied
            },
            None,
            MAX_BYTES
        )
        .unwrap()
        .cells(),
        &[false]
    );
    let four = CaRules2D {
        neighborhood: CaNeighborhood2D::VonNeumann4,
        birth_at_least: 4,
        survive_at_least: 4,
        outside_read: CaOutsideRead::Occupied,
        protected_border: 0,
    };
    assert_eq!(
        cellular_automata_step_2d(&input, four, None, MAX_BYTES)
            .unwrap()
            .cells(),
        &[true]
    );
}

#[test]
fn rejects_unsafe_sizes_regions_budgets_and_rules_before_execution() {
    let config = NoiseSample2D::legacy(42, 0.1, GridCoord2::ZERO);
    assert_eq!(
        sample_noise_2d(GridSize2::new(0, 64), config, MAX_BYTES),
        Err(GenerationError::EmptySize)
    );
    assert_eq!(
        sample_noise_2d(GridSize2::new(64, 64), config, 100),
        Err(GenerationError::ExceedsOutputBudget)
    );
    assert_eq!(
        sample_noise_2d(
            GridSize2::new(2, 2),
            NoiseSample2D::legacy(1, 0.1, GridCoord2::new(i32::MAX, 0)),
            MAX_BYTES,
        ),
        Err(GenerationError::RegionOutOfRange)
    );
    assert_eq!(
        sample_noise_2d(
            GridSize2::new(2, 2),
            NoiseSample2D::legacy(1, f32::NAN, GridCoord2::ZERO),
            MAX_BYTES,
        ),
        Err(GenerationError::InvalidFrequency)
    );
    let input = DenseGrid2D::new(2, 2, false);
    assert_eq!(
        cellular_automata_step_2d(&input, CaRules2D::legacy(), None, 0),
        Err(GenerationError::ExceedsOutputBudget)
    );
    assert_eq!(
        cellular_automata_step_2d(
            &input,
            CaRules2D::legacy(),
            Some(&DenseGrid2D::new(1, 1, true)),
            MAX_BYTES,
        ),
        Err(GenerationError::ProtectionSizeMismatch)
    );
    assert_eq!(
        cellular_automata_step_2d(
            &input,
            CaRules2D {
                birth_at_least: 10,
                ..CaRules2D::legacy()
            },
            None,
            MAX_BYTES,
        ),
        Err(GenerationError::InvalidRule)
    );
}
