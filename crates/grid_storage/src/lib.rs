use std::collections::BTreeMap;

pub use grid_math::{
    ALL_NEIGHBORS_2D, GridBox, GridCoord2, GridCoord3, GridRect, GridSize2, GridSize3,
    ORTHOGONAL_NEIGHBORS_2D, coord_from_row_major_index, row_major_index, row_major_index_3d,
};

pub trait GridRead {
    type Cell;

    fn size(&self) -> GridSize2;
    fn get(&self, coord: GridCoord2) -> Option<&Self::Cell>;

    fn contains(&self, coord: GridCoord2) -> bool {
        self.get(coord).is_some()
    }
}

pub trait GridWrite: GridRead {
    fn get_mut(&mut self, coord: GridCoord2) -> Option<&mut Self::Cell>;
    fn set(&mut self, coord: GridCoord2, value: Self::Cell) -> bool;
}

pub trait GridBounds {
    fn min(&self) -> GridCoord2;
    fn max_exclusive(&self) -> GridCoord2;

    fn rect(&self) -> GridRect {
        let min = self.min();
        let max = self.max_exclusive();

        GridRect::new(
            min,
            GridSize2::new((max.x - min.x) as usize, (max.y - min.y) as usize),
        )
    }

    fn max_inclusive(&self) -> Option<GridCoord2> {
        let max = self.max_exclusive();
        if max.x <= self.min().x || max.y <= self.min().y {
            return None;
        }

        Some(GridCoord2::new(max.x - 1, max.y - 1))
    }
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
        Self {
            size,
            cells: vec![default; size.area()],
        }
    }
}

impl<T> DenseGrid2D<T> {
    pub fn from_fn<F>(size: GridSize2, mut f: F) -> Self
    where
        F: FnMut(GridCoord2) -> T,
    {
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
        GridCoord2::new(self.width() as i32, self.height() as i32)
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

    pub fn set(&mut self, coord: GridCoord2, value: T) -> Option<T> {
        self.cells.insert(coord, value)
    }

    pub fn remove(&mut self, coord: GridCoord2) -> Option<T> {
        self.cells.remove(&coord)
    }

    pub fn iter_cells(&self) -> impl Iterator<Item = (GridCoord2, &T)> {
        self.cells.iter().map(|(coord, cell)| (*coord, cell))
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

    pub fn set(&mut self, coord: GridCoord3, value: T) -> Option<T> {
        self.cells.insert(coord, value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChunkedGrid2D<T> {
    chunk_size: GridSize2,
    chunks: BTreeMap<GridCoord2, DenseGrid2D<T>>,
}

impl<T> ChunkedGrid2D<T> {
    pub fn new(chunk_size: GridSize2) -> Self {
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
}
