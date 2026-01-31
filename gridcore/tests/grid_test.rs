// src/tests/grid_test.rs
use glam::IVec2;
use gridcore::*;
use crate::grid2d::CellType::{Wall as W, Empty as E};
use crate::grid2d::{CornerType};

#[test]
fn test_grid_macro_creation() {
    let g = grid![
        [W, E],
        [E, W],
    ];

    assert_eq!(g.width, 2);
    assert_eq!(g.height, 2);

    assert_eq!(g.get(IVec2::new(0, 0)), Some(&W));
    assert_eq!(g.get(IVec2::new(1, 0)), Some(&E));
    assert_eq!(g.get(IVec2::new(1, 1)), Some(&W));
}

#[test]
#[should_panic(expected = "All rows in grid! must have the same length")]
fn test_grid_macro_row_mismatch() {
    let _g = grid![
        [W, E],
        [W],
    ];
}

#[test]
fn test_contains_and_index_of() {
    let g = grid![
        [W, E],
        [E, W],
    ];

    assert!(g.contains(IVec2::new(0, 0)));
    assert!(g.contains(IVec2::new(1, 1)));
    assert!(!g.contains(IVec2::new(-1, 0)));
    assert!(!g.contains(IVec2::new(2, 0)));
}

#[test]
fn test_corner_mask_and_tiles() {
    let g = grid![
        [W, W],
        [W, W],
    ];

    assert_eq!(g.corner_mask(IVec2::new(0, 0)), 0b0001);

    let corners = g.corner_tiles(|mask, logic| {
        match mask {
            0b0000 => CornerType::Empty,
            0b1111 => CornerType::Full,
            0b0101 | 0b1010 | 0b0110 | 0b0011 | 0b1001 | 0b1100 => CornerType::Edge,
            _ => CornerType::Corner,
        }
    });

    assert_eq!(corners.len(), 3);
    assert_eq!(corners[0].len(), 3);

    let center = &corners[1][1];
    assert_eq!(center.visual, CornerType::Full);
    assert!(center.logic);
}

#[test]
fn test_for_each_cell() {
    let g = grid![
        [W, E],
        [E, W],
    ];

    let mut wall_count = 0;
    g.for_each_cell(|_coord, cell| {
        if *cell == W { wall_count += 1; }
    });

    assert_eq!(wall_count, 2);
}

#[test]
fn test_for_each_corner() {
    let g = grid![
        [W, W],
        [W, W],
    ];

    let mut full_count = 0;
    let corners = g.corner_tiles(|mask, _logic| {
        match mask {
            0b1111 => CornerType::Full,
            _ => CornerType::Empty,
        }
    });

    for row in &corners {
        for corner in row {
            if corner.visual == CornerType::Full { full_count += 1; }
        }
    }

    assert_eq!(full_count, 1); // only center
}
