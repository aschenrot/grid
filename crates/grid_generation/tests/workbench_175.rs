//! Research-only proof for Riesel #175; no new public grid/CA API.
//! Uses the real existing grid topology and noise-fill functions.
//! The configurable CA rule below is a test-local *candidate*, not production.
#![forbid(unsafe_code)]

use grid_generation::{NoiseFillSettings, fill_with_noise};
use grid_math::{ALL_NEIGHBORS_2D, GridCoord2, GridSize2};
use grid_storage::DenseGrid2D;
use tile_topology::{CellType, form_visual_tiles};

#[derive(Clone, Copy)]
enum OutsideRead {
    Empty,
    Occupied,
}

#[derive(Clone, Copy)]
struct CaRule {
    birth_min: usize,
    survive_min: usize,
    protected_border: usize,
    outside_read: OutsideRead,
}

fn border(coord: GridCoord2, size: GridSize2, thickness: usize) -> bool {
    let x = usize::try_from(coord.x).unwrap();
    let y = usize::try_from(coord.y).unwrap();
    x < thickness
        || y < thickness
        || x >= size.width.saturating_sub(thickness)
        || y >= size.height.saturating_sub(thickness)
}

/// Deliberately independent research rule using a read-only source grid:
/// new cells are computed from the previous iteration, never partly updated.
fn research_ca_step(input: &DenseGrid2D<CellType>, rule: CaRule) -> DenseGrid2D<CellType> {
    DenseGrid2D::from_fn(input.size(), |coord| {
        let previous = *input.get(coord).unwrap();
        if border(coord, input.size(), rule.protected_border) {
            return previous;
        }

        let occupied_neighbors = ALL_NEIGHBORS_2D.iter().filter(|offset| {
            match input.get(coord + **offset) {
                Some(CellType::Wall) => true,
                Some(CellType::Empty) => false,
                None => matches!(rule.outside_read, OutsideRead::Occupied),
            }
        }).count();
        let threshold = match previous {
            CellType::Wall => rule.survive_min,
            CellType::Empty => rule.birth_min,
        };
        if occupied_neighbors >= threshold {
            CellType::Wall
        } else {
            CellType::Empty
        }
    })
}

#[test]
fn real_noise_fill_configurable_ca_preserves_mixed_border_after_every_step() {
    const SIZE: usize = 64;
    let mut input = DenseGrid2D::new(SIZE, SIZE, CellType::Empty);
    fill_with_noise(&mut input, NoiseFillSettings::new(0.1, 0.55, 42));

    // Force a mixed border: preservation means keep original values, not "all walls".
    for (coord, cell) in input.iter_cells_mut() {
        if border(coord, GridSize2::new(SIZE, SIZE), 1) {
            *cell = if (coord.x + coord.y) % 2 == 0 {
                CellType::Wall
            } else {
                CellType::Empty
            };
        }
    }
    let original = input.clone();

    let rule = CaRule {
        birth_min: 5,
        survive_min: 4,
        protected_border: 1,
        outside_read: OutsideRead::Occupied,
    };
    let mut current = input;
    for _iteration in 1..=4 {
        current = research_ca_step(&current, rule);
        for (coord, cell) in current.iter_cells() {
            if border(coord, current.size(), 1) {
                assert_eq!(Some(cell), original.get(coord));
            }
        }
    }

    let visual = form_visual_tiles(&current);
    assert_eq!((current.width(), current.height()), (64, 64));
    assert_eq!((visual.width(), visual.height()), (65, 65));

    // Same input, rule and iteration count are reproducible.
    let mut replay = original.clone();
    for _ in 0..4 {
        replay = research_ca_step(&replay, rule);
    }
    assert_eq!(replay, current);
}

#[test]
fn different_out_of_bounds_read_policies_are_observable() {
    // With no protected cells, all eight neighbors of the 1x1 grid are OOB.
    let input = DenseGrid2D::new(1, 1, CellType::Empty);
    let base = CaRule {
        birth_min: 8,
        survive_min: 8,
        protected_border: 0,
        outside_read: OutsideRead::Empty,
    };
    let empty = research_ca_step(&input, base);
    let occupied = research_ca_step(
        &input,
        CaRule { outside_read: OutsideRead::Occupied, ..base },
    );
    assert_eq!(empty.get(GridCoord2::new(0, 0)), Some(&CellType::Empty));
    assert_eq!(occupied.get(GridCoord2::new(0, 0)), Some(&CellType::Wall));
}

#[test]
fn all_sixteen_masks_use_real_upstream_topology() {
    for mask in 0u8..16 {
        let logic = DenseGrid2D::from_fn(GridSize2::new(2, 2), |coord| {
            let bit = (coord.x + coord.y * 2) as u32;
            if mask & (1u8 << bit) != 0 {
                CellType::Wall
            } else {
                CellType::Empty
            }
        });
        let visual = form_visual_tiles(&logic);
        let tile = visual.get(GridCoord2::new(1, 1)).unwrap();
        assert_eq!(tile.descriptor().mask().bits(), mask);
        assert_eq!((visual.width(), visual.height()), (3, 3));
    }
}

#[test]
fn scalar_sample_grid_and_occupancy_grid_are_distinct_typed_values() {
    let scalar = DenseGrid2D::from_fn(GridSize2::new(2, 2), |coord| {
        f64::from(coord.x + coord.y * 2) / 3.0
    });
    let occupancy = DenseGrid2D::from_fn(scalar.size(), |coord| {
        if *scalar.get(coord).unwrap() > 0.5 {
            CellType::Wall
        } else {
            CellType::Empty
        }
    });
    assert_eq!(occupancy.get(GridCoord2::new(0, 0)), Some(&CellType::Empty));
    assert_eq!(occupancy.get(GridCoord2::new(1, 0)), Some(&CellType::Empty));
    assert_eq!(occupancy.get(GridCoord2::new(0, 1)), Some(&CellType::Wall));
    assert_eq!(occupancy.get(GridCoord2::new(1, 1)), Some(&CellType::Wall));
}
