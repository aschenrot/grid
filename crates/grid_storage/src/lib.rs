use std::collections::BTreeMap;

pub use grid_math::{
    ALL_NEIGHBORS_2D, GridBox, GridCoord2, GridCoord3, GridRect, GridSize2, GridSize3,
    ORTHOGONAL_NEIGHBORS_2D, coord_from_row_major_index, row_major_index, row_major_index_3d,
};

/// Read access for finite, zero-origin, rectangular 2D grids.
///
/// Coordinates accepted by this trait are local to the grid bounds:
/// `0..size.width` and `0..size.height`. World-space or sparse containers with
/// negative/offset coordinates should expose explicit accessors and
/// [`GridBounds`] instead of pretending to be zero-origin dense storage.
pub trait GridRead {
    type Cell;

    fn size(&self) -> GridSize2;
    fn get(&self, coord: GridCoord2) -> Option<&Self::Cell>;

    fn contains(&self, coord: GridCoord2) -> bool {
        self.get(coord).is_some()
    }
}

/// Write access for finite, zero-origin, rectangular 2D grids.
pub trait GridWrite: GridRead {
    fn get_mut(&mut self, coord: GridCoord2) -> Option<&mut Self::Cell>;
    fn set(&mut self, coord: GridCoord2, value: Self::Cell) -> bool;
}

/// World-space 2D bounds for storage that can report an occupied extent.
pub trait GridBounds {
    fn min(&self) -> GridCoord2;
    fn max_exclusive(&self) -> GridCoord2;

    fn try_rect(&self) -> Option<GridRect> {
        let min = self.min();
        let max = self.max_exclusive();
        let width = usize::try_from(max.x.checked_sub(min.x)?).ok()?;
        let height = usize::try_from(max.y.checked_sub(min.y)?).ok()?;

        Some(GridRect::new(min, GridSize2::new(width, height)))
    }

    fn rect(&self) -> GridRect {
        self.try_rect()
            .expect("grid bounds max_exclusive must be greater than or equal to min")
    }

    fn max_inclusive(&self) -> Option<GridCoord2> {
        let max = self.max_exclusive();
        if max.x <= self.min().x || max.y <= self.min().y {
            return None;
        }

        Some(GridCoord2::new(max.x - 1, max.y - 1))
    }
}

/// Why a bounded caller could not construct a finite dense grid.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum DenseGridBuildError {
    CoordinateDimensionsOutOfRange,
    CellCountOverflow,
    CellCountMismatch { expected: usize, actual: usize },
    AllocationFailed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DenseGrid2D<T> {
    size: GridSize2,
    cells: Vec<T>,
}

impl<T: Clone> DenseGrid2D<T> {
    pub fn new(width: usize, height: usize, default: T) -> Self {
        Self::filled(GridSize2::new(width, height), default)
    }

    pub fn filled(size: GridSize2, default: T) -> Self {
        assert!(
            size.fits_i32_coords(),
            "DenseGrid2D dimensions must fit in i32 coordinates"
        );

        Self {
            size,
            cells: vec![default; size.area()],
        }
    }
}

impl<T> DenseGrid2D<T> {
    /// Takes an already-owned row-major buffer without reallocating.
    ///
    /// This avoids implicit panics from dimensions or an incorrect cell count;
    /// the supplied buffer remains owned by the caller on error only through
    /// the returned error classification (it is dropped on failure).
    pub fn try_from_vec(size: GridSize2, cells: Vec<T>) -> Result<Self, DenseGridBuildError> {
        if !size.fits_i32_coords() {
            return Err(DenseGridBuildError::CoordinateDimensionsOutOfRange);
        }
        let expected = size
            .checked_area()
            .ok_or(DenseGridBuildError::CellCountOverflow)?;
        if cells.len() != expected {
            return Err(DenseGridBuildError::CellCountMismatch {
                expected,
                actual: cells.len(),
            });
        }
        Ok(Self { size, cells })
    }

    /// Builds a dense grid with a fallible reservation before invoking `f`.
    ///
    /// It does not catch panics in user callbacks or guarantee availability of
    /// physical memory on systems with overcommit.
    pub fn try_from_fn<F>(
        size: GridSize2,
        mut f: F,
    ) -> Result<Self, DenseGridBuildError>
    where
        F: FnMut(GridCoord2) -> T,
    {
        if !size.fits_i32_coords() {
            return Err(DenseGridBuildError::CoordinateDimensionsOutOfRange);
        }
        let count = size
            .checked_area()
            .ok_or(DenseGridBuildError::CellCountOverflow)?;
        let mut cells = Vec::new();
        cells
            .try_reserve_exact(count)
            .map_err(|_| DenseGridBuildError::AllocationFailed)?;

        for y in 0..size.height {
            for x in 0..size.width {
                cells.push(f(GridCoord2::new(x as i32, y as i32)));
            }
        }

        Ok(Self { size, cells })
    }

    pub fn from_fn<F>(size: GridSize2, mut f: F) -> Self
    where
        F: FnMut(GridCoord2) -> T,
    {
        assert!(
            size.fits_i32_coords(),
            "DenseGrid2D dimensions must fit in i32 coordinates"
        );

        let cells = GridRect::from_size(size).iter().map(&mut f).collect();

        Self { size, cells }
    }

    pub const fn size(&self) -> GridSize2 {
        self.size
    }

    pub const fn width(&self) -> usize {
        self.size.width
    }

    pub const fn height(&self) -> usize {
        self.size.height
    }

    pub fn len(&self) -> usize {
        self.cells.len()
    }

    pub fn is_empty(&self) -> bool {
        self.cells.is_empty()
    }

    pub fn cells(&self) -> &[T] {
        &self.cells
    }

    pub fn cells_mut(&mut self) -> &mut [T] {
        &mut self.cells
    }

    pub fn index_of(&self, coord: GridCoord2) -> Option<usize> {
        row_major_index(coord, self.size)
    }

    pub fn coord_of_index(&self, index: usize) -> Option<GridCoord2> {
        coord_from_row_major_index(index, self.size)
    }

    pub fn get(&self, coord: GridCoord2) -> Option<&T> {
        self.index_of(coord).map(|index| &self.cells[index])
    }

    pub fn get_mut(&mut self, coord: GridCoord2) -> Option<&mut T> {
        self.index_of(coord).map(|index| &mut self.cells[index])
    }

    pub fn set(&mut self, coord: GridCoord2, value: T) -> bool {
        if let Some(index) = self.index_of(coord) {
            self.cells[index] = value;
            true
        } else {
            false
        }
    }

    pub fn contains(&self, coord: GridCoord2) -> bool {
        self.index_of(coord).is_some()
    }

    pub fn iter_cells(&self) -> impl Iterator<Item = (GridCoord2, &T)> {
        let size = self.size;
        self.cells
            .iter()
            .enumerate()
            .filter_map(move |(index, cell)| {
                coord_from_row_major_index(index, size).map(|c| (c, cell))
            })
    }

    pub fn iter_cells_mut(&mut self) -> impl Iterator<Item = (GridCoord2, &mut T)> {
        let size = self.size;
        self.cells
            .iter_mut()
            .enumerate()
            .filter_map(move |(index, cell)| {
                coord_from_row_major_index(index, size).map(|coord| (coord, cell))
            })
    }

    pub fn neighbors_8(&self, coord: GridCoord2) -> Vec<GridCoord2> {
        ALL_NEIGHBORS_2D
            .iter()
            .map(|offset| coord + *offset)
            .filter(|neighbor| self.contains(*neighbor))
            .collect()
    }

    pub fn neighbors_4(&self, coord: GridCoord2) -> Vec<GridCoord2> {
        ORTHOGONAL_NEIGHBORS_2D
            .iter()
            .map(|offset| coord + *offset)
            .filter(|neighbor| self.contains(*neighbor))
            .collect()
    }
}

impl<T> GridRead for DenseGrid2D<T> {
    type Cell = T;

    fn size(&self) -> GridSize2 {
        self.size
    }

    fn get(&self, coord: GridCoord2) -> Option<&Self::Cell> {
        DenseGrid2D::get(self, coord)
    }

    fn contains(&self, coord: GridCoord2) -> bool {
        DenseGrid2D::contains(self, coord)
    }
}

impl<T> GridWrite for DenseGrid2D<T> {
    fn get_mut(&mut self, coord: GridCoord2) -> Option<&mut Self::Cell> {
        DenseGrid2D::get_mut(self, coord)
    }

    fn set(&mut self, coord: GridCoord2, value: Self::Cell) -> bool {
        DenseGrid2D::set(self, coord, value)
    }
}

impl<T> GridBounds for DenseGrid2D<T> {
    fn min(&self) -> GridCoord2 {
        GridCoord2::ZERO
    }

    fn max_exclusive(&self) -> GridCoord2 {
        GridCoord2::new(
            i32::try_from(self.width()).expect("grid width exceeds i32::MAX"),
            i32::try_from(self.height()).expect("grid height exceeds i32::MAX"),
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DenseGrid3D<T> {
    size: GridSize3,
    cells: Vec<T>,
}

impl<T: Clone> DenseGrid3D<T> {
    pub fn new(width: usize, height: usize, depth: usize, default: T) -> Self {
        Self::filled(GridSize3::new(width, height, depth), default)
    }

    pub fn filled(size: GridSize3, default: T) -> Self {
        assert!(
            size.fits_i32_coords(),
            "DenseGrid3D dimensions must fit in i32 coordinates"
        );

        Self {
            size,
            cells: vec![default; size.volume()],
        }
    }
}

impl<T> DenseGrid3D<T> {
    pub const fn size(&self) -> GridSize3 {
        self.size
    }

    pub fn index_of(&self, coord: GridCoord3) -> Option<usize> {
        row_major_index_3d(coord, self.size)
    }

    pub fn get(&self, coord: GridCoord3) -> Option<&T> {
        self.index_of(coord).map(|index| &self.cells[index])
    }

    pub fn get_mut(&mut self, coord: GridCoord3) -> Option<&mut T> {
        self.index_of(coord).map(|index| &mut self.cells[index])
    }

    pub fn set(&mut self, coord: GridCoord3, value: T) -> bool {
        if let Some(index) = self.index_of(coord) {
            self.cells[index] = value;
            true
        } else {
            false
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SparseGrid2D<T> {
    cells: BTreeMap<GridCoord2, T>,
}

impl<T> SparseGrid2D<T> {
    pub fn new() -> Self {
        Self {
            cells: BTreeMap::new(),
        }
    }

    pub fn get(&self, coord: GridCoord2) -> Option<&T> {
        self.cells.get(&coord)
    }

    pub fn get_mut(&mut self, coord: GridCoord2) -> Option<&mut T> {
        self.cells.get_mut(&coord)
    }

    pub fn set(&mut self, coord: GridCoord2, value: T) -> Option<T> {
        self.cells.insert(coord, value)
    }

    pub fn remove(&mut self, coord: GridCoord2) -> Option<T> {
        self.cells.remove(&coord)
    }

    pub fn contains(&self, coord: GridCoord2) -> bool {
        self.cells.contains_key(&coord)
    }

    pub fn len(&self) -> usize {
        self.cells.len()
    }

    pub fn is_empty(&self) -> bool {
        self.cells.is_empty()
    }

    pub fn clear(&mut self) {
        self.cells.clear();
    }

    pub fn bounds_rect(&self) -> Option<GridRect> {
        self.try_rect()
    }

    pub fn iter_cells(&self) -> impl Iterator<Item = (GridCoord2, &T)> {
        self.cells.iter().map(|(coord, cell)| (*coord, cell))
    }
}

impl<T> GridBounds for SparseGrid2D<T> {
    fn min(&self) -> GridCoord2 {
        let mut iter = self.cells.keys();
        let Some(first) = iter.next() else {
            return GridCoord2::ZERO;
        };

        iter.fold(*first, |min, coord| {
            GridCoord2::new(min.x.min(coord.x), min.y.min(coord.y))
        })
    }

    fn max_exclusive(&self) -> GridCoord2 {
        let mut iter = self.cells.keys();
        let Some(first) = iter.next() else {
            return GridCoord2::ZERO;
        };

        let max_inclusive = iter.fold(*first, |max, coord| {
            GridCoord2::new(max.x.max(coord.x), max.y.max(coord.y))
        });

        max_inclusive.offset(1, 1)
    }

    fn try_rect(&self) -> Option<GridRect> {
        if self.is_empty() {
            return None;
        }

        let min = self.min();
        let max = self.max_exclusive();
        let width = usize::try_from(max.x.checked_sub(min.x)?).ok()?;
        let height = usize::try_from(max.y.checked_sub(min.y)?).ok()?;

        Some(GridRect::new(min, GridSize2::new(width, height)))
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SparseGrid3D<T> {
    cells: BTreeMap<GridCoord3, T>,
}

impl<T> SparseGrid3D<T> {
    pub fn new() -> Self {
        Self {
            cells: BTreeMap::new(),
        }
    }

    pub fn get(&self, coord: GridCoord3) -> Option<&T> {
        self.cells.get(&coord)
    }

    pub fn get_mut(&mut self, coord: GridCoord3) -> Option<&mut T> {
        self.cells.get_mut(&coord)
    }

    pub fn set(&mut self, coord: GridCoord3, value: T) -> Option<T> {
        self.cells.insert(coord, value)
    }

    pub fn remove(&mut self, coord: GridCoord3) -> Option<T> {
        self.cells.remove(&coord)
    }

    pub fn contains(&self, coord: GridCoord3) -> bool {
        self.cells.contains_key(&coord)
    }

    pub fn len(&self) -> usize {
        self.cells.len()
    }

    pub fn is_empty(&self) -> bool {
        self.cells.is_empty()
    }

    pub fn clear(&mut self) {
        self.cells.clear();
    }

    pub fn iter_cells(&self) -> impl Iterator<Item = (GridCoord3, &T)> {
        self.cells.iter().map(|(coord, cell)| (*coord, cell))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChunkedGrid2D<T> {
    chunk_size: GridSize2,
    chunks: BTreeMap<GridCoord2, DenseGrid2D<T>>,
}

impl<T> ChunkedGrid2D<T> {
    pub fn new(chunk_size: GridSize2) -> Self {
        assert!(
            chunk_size.width > 0 && chunk_size.height > 0,
            "chunk size must be non-zero"
        );
        assert!(
            chunk_size.fits_i32_coords(),
            "chunk dimensions must fit in i32 coordinates"
        );

        Self {
            chunk_size,
            chunks: BTreeMap::new(),
        }
    }

    pub const fn chunk_size(&self) -> GridSize2 {
        self.chunk_size
    }

    pub fn chunks(&self) -> &BTreeMap<GridCoord2, DenseGrid2D<T>> {
        &self.chunks
    }

    pub fn len_chunks(&self) -> usize {
        self.chunks.len()
    }

    pub fn is_empty(&self) -> bool {
        self.chunks.is_empty()
    }

    pub fn clear(&mut self) {
        self.chunks.clear();
    }

    pub fn chunk_coord_for(&self, coord: GridCoord2) -> GridCoord2 {
        GridCoord2::new(
            div_floor(coord.x, usize_to_i32(self.chunk_size.width)),
            div_floor(coord.y, usize_to_i32(self.chunk_size.height)),
        )
    }

    pub fn local_coord_for(&self, coord: GridCoord2) -> GridCoord2 {
        let chunk_coord = self.chunk_coord_for(coord);
        GridCoord2::new(
            coord.x - chunk_coord.x * usize_to_i32(self.chunk_size.width),
            coord.y - chunk_coord.y * usize_to_i32(self.chunk_size.height),
        )
    }

    pub fn get(&self, coord: GridCoord2) -> Option<&T> {
        let chunk_coord = self.chunk_coord_for(coord);
        let local_coord = self.local_coord_for(coord);

        self.chunks
            .get(&chunk_coord)
            .and_then(|chunk| chunk.get(local_coord))
    }

    pub fn get_mut(&mut self, coord: GridCoord2) -> Option<&mut T> {
        let chunk_coord = self.chunk_coord_for(coord);
        let local_coord = self.local_coord_for(coord);

        self.chunks
            .get_mut(&chunk_coord)
            .and_then(|chunk| chunk.get_mut(local_coord))
    }

    pub fn contains_chunk(&self, chunk_coord: GridCoord2) -> bool {
        self.chunks.contains_key(&chunk_coord)
    }

    pub fn remove_chunk(&mut self, chunk_coord: GridCoord2) -> Option<DenseGrid2D<T>> {
        self.chunks.remove(&chunk_coord)
    }

    pub fn iter_chunks(&self) -> impl Iterator<Item = (GridCoord2, &DenseGrid2D<T>)> {
        self.chunks.iter().map(|(coord, chunk)| (*coord, chunk))
    }

    pub fn iter_cells(&self) -> impl Iterator<Item = (GridCoord2, &T)> {
        let chunk_size = self.chunk_size;
        self.chunks.iter().flat_map(move |(chunk_coord, chunk)| {
            chunk.iter_cells().map(move |(local_coord, cell)| {
                (
                    global_coord_from_chunk(*chunk_coord, local_coord, chunk_size),
                    cell,
                )
            })
        })
    }
}

impl<T> GridBounds for ChunkedGrid2D<T> {
    fn min(&self) -> GridCoord2 {
        let mut iter = self.chunks.keys();
        let Some(first) = iter.next() else {
            return GridCoord2::ZERO;
        };

        iter.fold(chunk_origin(*first, self.chunk_size), |min, chunk_coord| {
            let origin = chunk_origin(*chunk_coord, self.chunk_size);
            GridCoord2::new(min.x.min(origin.x), min.y.min(origin.y))
        })
    }

    fn max_exclusive(&self) -> GridCoord2 {
        let mut iter = self.chunks.keys();
        let Some(first) = iter.next() else {
            return GridCoord2::ZERO;
        };

        iter.fold(
            chunk_max_exclusive(*first, self.chunk_size),
            |max, chunk_coord| {
                let chunk_max = chunk_max_exclusive(*chunk_coord, self.chunk_size);
                GridCoord2::new(max.x.max(chunk_max.x), max.y.max(chunk_max.y))
            },
        )
    }

    fn try_rect(&self) -> Option<GridRect> {
        if self.is_empty() {
            return None;
        }

        let min = self.min();
        let max = self.max_exclusive();
        let width = usize::try_from(max.x.checked_sub(min.x)?).ok()?;
        let height = usize::try_from(max.y.checked_sub(min.y)?).ok()?;

        Some(GridRect::new(min, GridSize2::new(width, height)))
    }
}

impl<T: Clone> ChunkedGrid2D<T> {
    pub fn ensure_chunk(&mut self, chunk_coord: GridCoord2, default: T) -> &mut DenseGrid2D<T> {
        self.chunks
            .entry(chunk_coord)
            .or_insert_with(|| DenseGrid2D::filled(self.chunk_size, default))
    }

    pub fn set(&mut self, coord: GridCoord2, value: T, default: T) -> Option<T> {
        let chunk_coord = self.chunk_coord_for(coord);
        let local_coord = self.local_coord_for(coord);
        let chunk = self.ensure_chunk(chunk_coord, default);
        let old = chunk.get(local_coord).cloned();
        chunk.set(local_coord, value);
        old
    }
}

fn div_floor(value: i32, divisor: i32) -> i32 {
    debug_assert!(divisor > 0);
    let quotient = value / divisor;
    let remainder = value % divisor;
    if remainder != 0 && ((remainder < 0) != (divisor < 0)) {
        quotient - 1
    } else {
        quotient
    }
}

fn usize_to_i32(value: usize) -> i32 {
    i32::try_from(value).expect("dimension exceeds i32::MAX")
}

fn global_coord_from_chunk(
    chunk_coord: GridCoord2,
    local_coord: GridCoord2,
    chunk_size: GridSize2,
) -> GridCoord2 {
    let origin = chunk_origin(chunk_coord, chunk_size);
    GridCoord2::new(origin.x + local_coord.x, origin.y + local_coord.y)
}

fn chunk_origin(chunk_coord: GridCoord2, chunk_size: GridSize2) -> GridCoord2 {
    GridCoord2::new(
        chunk_coord
            .x
            .checked_mul(usize_to_i32(chunk_size.width))
            .expect("chunk origin x overflows i32"),
        chunk_coord
            .y
            .checked_mul(usize_to_i32(chunk_size.height))
            .expect("chunk origin y overflows i32"),
    )
}

fn chunk_max_exclusive(chunk_coord: GridCoord2, chunk_size: GridSize2) -> GridCoord2 {
    let origin = chunk_origin(chunk_coord, chunk_size);
    GridCoord2::new(
        origin
            .x
            .checked_add(usize_to_i32(chunk_size.width))
            .expect("chunk max x overflows i32"),
        origin
            .y
            .checked_add(usize_to_i32(chunk_size.height))
            .expect("chunk max y overflows i32"),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dense_grid_rejects_negative_and_out_of_range_coords() {
        let grid = DenseGrid2D::new(2, 2, 0);

        assert_eq!(grid.get(GridCoord2::new(-1, 0)), None);
        assert_eq!(grid.get(GridCoord2::new(0, -1)), None);
        assert_eq!(grid.get(GridCoord2::new(2, 0)), None);
        assert_eq!(grid.get(GridCoord2::new(0, 2)), None);
    }

    #[test]
    fn dense_grid_sets_and_reads_cells() {
        let mut grid = DenseGrid2D::new(3, 2, 0);

        assert!(grid.set(GridCoord2::new(2, 1), 7));
        assert_eq!(grid.get(GridCoord2::new(2, 1)), Some(&7));
        assert_eq!(grid.index_of(GridCoord2::new(2, 1)), Some(5));
        assert_eq!(grid.coord_of_index(5), Some(GridCoord2::new(2, 1)));
        assert!(!grid.set(GridCoord2::new(3, 1), 9));
    }

    #[test]
    fn dense_grid_iterates_in_row_major_order() {
        let grid = DenseGrid2D::from_fn(GridSize2::new(2, 2), |coord| coord.x + coord.y * 10);
        let values: Vec<_> = grid.iter_cells().collect();

        assert_eq!(
            values,
            vec![
                (GridCoord2::new(0, 0), &0),
                (GridCoord2::new(1, 0), &1),
                (GridCoord2::new(0, 1), &10),
                (GridCoord2::new(1, 1), &11),
            ]
        );
    }

    #[test]
    fn dense_grid_neighbors_stay_in_bounds() {
        let grid = DenseGrid2D::new(3, 3, 0);

        assert_eq!(
            grid.neighbors_4(GridCoord2::new(0, 0)),
            vec![GridCoord2::new(1, 0), GridCoord2::new(0, 1)]
        );
        assert_eq!(grid.neighbors_8(GridCoord2::new(1, 1)).len(), 8);
    }

    #[test]
    fn dense_grid_3d_indexes_rows_then_layers() {
        let mut grid = DenseGrid3D::new(2, 2, 2, 0);

        assert!(grid.set(GridCoord3::new(0, 0, 1), 5));
        assert_eq!(grid.index_of(GridCoord3::new(0, 0, 1)), Some(4));
        assert_eq!(grid.get(GridCoord3::new(0, 0, 1)), Some(&5));
        assert_eq!(grid.get(GridCoord3::new(-1, 0, 0)), None);
    }

    #[test]
    fn sparse_grid_2d_tracks_bounds_and_traits() {
        let mut grid = SparseGrid2D::new();

        assert!(grid.is_empty());
        assert_eq!(grid.set(GridCoord2::new(-2, 3), "a"), None);
        assert_eq!(grid.set(GridCoord2::new(4, -1), "b"), None);

        assert_eq!(grid.len(), 2);
        assert_eq!(grid.get(GridCoord2::new(-2, 3)), Some(&"a"));
        assert_eq!(grid.min(), GridCoord2::new(-2, -1));
        assert_eq!(grid.max_exclusive(), GridCoord2::new(5, 4));
        assert_eq!(
            grid.bounds_rect(),
            Some(GridRect::new(GridCoord2::new(-2, -1), GridSize2::new(7, 5)))
        );
        assert_eq!(grid.remove(GridCoord2::new(-2, 3)), Some("a"));
        grid.clear();
        assert_eq!(grid.bounds_rect(), None);
    }

    #[test]
    fn sparse_grid_3d_sets_removes_and_iterates() {
        let mut grid = SparseGrid3D::new();

        grid.set(GridCoord3::new(1, 2, 3), 7);
        assert!(grid.contains(GridCoord3::new(1, 2, 3)));
        assert_eq!(grid.get_mut(GridCoord3::new(1, 2, 3)), Some(&mut 7));
        assert_eq!(
            grid.iter_cells().collect::<Vec<_>>(),
            vec![(GridCoord3::new(1, 2, 3), &7)]
        );
        assert_eq!(grid.remove(GridCoord3::new(1, 2, 3)), Some(7));
        assert!(grid.is_empty());
    }

    #[test]
    fn chunked_grid_maps_negative_coordinates_with_floor_division() {
        let mut grid = ChunkedGrid2D::new(GridSize2::new(4, 4));

        assert_eq!(
            grid.chunk_coord_for(GridCoord2::new(-1, -1)),
            GridCoord2::new(-1, -1)
        );
        assert_eq!(
            grid.local_coord_for(GridCoord2::new(-1, -1)),
            GridCoord2::new(3, 3)
        );

        assert_eq!(grid.set(GridCoord2::new(-1, -1), 9, 0), Some(0));
        assert_eq!(grid.get(GridCoord2::new(-1, -1)), Some(&9));
        assert!(grid.contains_chunk(GridCoord2::new(-1, -1)));
        assert_eq!(grid.len_chunks(), 1);
        assert_eq!(grid.min(), GridCoord2::new(-4, -4));
        assert_eq!(grid.max_exclusive(), GridCoord2::new(0, 0));
        assert_eq!(
            grid.iter_cells()
                .find(|(coord, _)| *coord == GridCoord2::new(-1, -1)),
            Some((GridCoord2::new(-1, -1), &9))
        );
    }
}

#[cfg(test)]
mod fallible_dense_construction_tests {
    use super::*;

    #[test]
    fn checked_construction_preserves_row_major_data() {
        let size = GridSize2::new(3, 2);
        let grid = DenseGrid2D::try_from_fn(size, |coord| coord.x + 10 * coord.y).unwrap();
        assert_eq!(grid.cells(), &[0, 1, 2, 10, 11, 12]);
        let rebuilt = DenseGrid2D::try_from_vec(size, grid.cells().to_vec()).unwrap();
        assert_eq!(rebuilt, grid);
    }

    #[test]
    fn checked_construction_rejects_invalid_shape() {
        assert_eq!(
            DenseGrid2D::<u8>::try_from_vec(GridSize2::new(2, 2), vec![1]),
            Err(DenseGridBuildError::CellCountMismatch {
                expected: 4,
                actual: 1
            })
        );
        assert_eq!(
            DenseGrid2D::<u8>::try_from_fn(GridSize2::new(i32::MAX as usize + 1, 1), |_| 0),
            Err(DenseGridBuildError::CoordinateDimensionsOutOfRange)
        );
        assert_eq!(
            DenseGrid2D::<u8>::try_from_fn(GridSize2::new(i32::MAX as usize, i32::MAX as usize), |_| 0),
            Err(DenseGridBuildError::AllocationFailed)
        );
    }
}
