//! Research-only characterization of public Grid boundary behavior.
//!
//! These tests deliberately assert *current failures* at legal i32 coordinates.
//! They must NOT be merged as production regression tests: a correctness fix
//! should replace each panic assertion with an expected successful result.
#![forbid(unsafe_code)]

use grid_math::{GridCoord2, GridRect, GridSize2, row_major_index};
use grid_storage::{ChunkedGrid2D, GridBounds, SparseGrid2D};
use std::panic::catch_unwind;

#[test]
#[cfg(debug_assertions)]
fn sparse_bounds_cannot_encode_a_cell_at_i32_max() {
    let mut grid = SparseGrid2D::new();
    grid.set(GridCoord2::new(i32::MAX, 0), 1u8);
    assert_eq!(grid.get(GridCoord2::new(i32::MAX, 0)), Some(&1));
    assert!(
        catch_unwind(|| grid.max_exclusive()).is_err(),
        "current max_exclusive adds one without overflow checking"
    );
}

#[test]
#[cfg(debug_assertions)]
fn empty_chunked_grid_get_can_panic_at_i32_min() {
    let grid = ChunkedGrid2D::<u8>::new(GridSize2::new(3, 3));
    assert!(grid.is_empty());
    assert!(
        catch_unwind(|| grid.get(GridCoord2::new(i32::MIN, 0))).is_err(),
        "current local_coord_for multiplies chunk coordinate by size in i32"
    );
}

#[test]
#[cfg(debug_assertions)]
fn rectangular_contains_can_panic_for_a_far_away_cell() {
    let rect = GridRect::new(GridCoord2::new(i32::MIN, 0), GridSize2::new(1, 1));
    assert!(rect.contains(GridCoord2::new(i32::MIN, 0)));
    assert!(
        catch_unwind(|| rect.contains(GridCoord2::new(i32::MAX, 0))).is_err(),
        "current GridRect::contains subtracts coordinates in i32 before bounds check"
    );
}

#[test]
#[cfg(debug_assertions)]
fn row_major_option_api_can_panic_instead_of_returning_none() {
    let oversized = GridSize2::new(usize::MAX, 2);
    assert_eq!(oversized.checked_area(), None);
    assert!(
        catch_unwind(|| row_major_index(GridCoord2::new(1, 1), oversized)).is_err(),
        "current row_major_index can overflow usize despite returning Option"
    );
}

#[test]
fn safe_range_reference_examples_still_work() {
    let mut sparse = SparseGrid2D::new();
    sparse.set(GridCoord2::new(-2, -1), 11u8);
    sparse.set(GridCoord2::new(4, 5), 12u8);
    assert_eq!(
        sparse.bounds_rect(),
        Some(GridRect::new(GridCoord2::new(-2, -1), GridSize2::new(7, 7),))
    );
    let chunked = ChunkedGrid2D::<u8>::new(GridSize2::new(3, 3));
    assert_eq!(chunked.get(GridCoord2::new(-7, -8)), None);
    assert_eq!(
        row_major_index(GridCoord2::new(1, 1), GridSize2::new(3, 3)),
        Some(4)
    );
}
