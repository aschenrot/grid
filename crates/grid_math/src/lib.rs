use core::ops::{Add, AddAssign, Sub, SubAssign};

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct GridCoord2 {
    pub x: i32,
    pub y: i32,
}

impl GridCoord2 {
    pub const ZERO: Self = Self::new(0, 0);

    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    pub const fn offset(self, dx: i32, dy: i32) -> Self {
        Self::new(self.x + dx, self.y + dy)
    }
}

impl Add for GridCoord2 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl AddAssign for GridCoord2 {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl Sub for GridCoord2 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl SubAssign for GridCoord2 {
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct GridCoord3 {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl GridCoord3 {
    pub const ZERO: Self = Self::new(0, 0, 0);

    pub const fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    pub const fn offset(self, dx: i32, dy: i32, dz: i32) -> Self {
        Self::new(self.x + dx, self.y + dy, self.z + dz)
    }
}

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, Hash)]
pub struct GridSize2 {
    pub width: usize,
    pub height: usize,
}

impl GridSize2 {
    pub const ZERO: Self = Self::new(0, 0);

    pub const fn new(width: usize, height: usize) -> Self {
        Self { width, height }
    }

    pub const fn checked_area(self) -> Option<usize> {
        self.width.checked_mul(self.height)
    }

    pub fn area(self) -> usize {
        self.checked_area().expect("grid area overflows usize")
    }

    pub fn fits_i32_coords(self) -> bool {
        i32::try_from(self.width).is_ok() && i32::try_from(self.height).is_ok()
    }

    pub const fn is_empty(self) -> bool {
        self.width == 0 || self.height == 0
    }

    pub fn contains(self, coord: GridCoord2) -> bool {
        row_major_index(coord, self).is_some()
    }
}

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, Hash)]
pub struct GridSize3 {
    pub width: usize,
    pub height: usize,
    pub depth: usize,
}

impl GridSize3 {
    pub const ZERO: Self = Self::new(0, 0, 0);

    pub const fn new(width: usize, height: usize, depth: usize) -> Self {
        Self {
            width,
            height,
            depth,
        }
    }

    pub const fn checked_volume(self) -> Option<usize> {
        match self.width.checked_mul(self.height) {
            Some(area) => area.checked_mul(self.depth),
            None => None,
        }
    }

    pub fn volume(self) -> usize {
        self.checked_volume().expect("grid volume overflows usize")
    }

    pub fn fits_i32_coords(self) -> bool {
        i32::try_from(self.width).is_ok()
            && i32::try_from(self.height).is_ok()
            && i32::try_from(self.depth).is_ok()
    }

    pub const fn is_empty(self) -> bool {
        self.width == 0 || self.height == 0 || self.depth == 0
    }

    pub fn contains(self, coord: GridCoord3) -> bool {
        row_major_index_3d(coord, self).is_some()
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct GridRect {
    pub origin: GridCoord2,
    pub size: GridSize2,
}

impl GridRect {
    pub const fn new(origin: GridCoord2, size: GridSize2) -> Self {
        Self { origin, size }
    }

    pub const fn from_size(size: GridSize2) -> Self {
        Self::new(GridCoord2::ZERO, size)
    }

    pub fn contains(self, coord: GridCoord2) -> bool {
        if coord.x < self.origin.x || coord.y < self.origin.y {
            return false;
        }

        let local = GridCoord2::new(coord.x - self.origin.x, coord.y - self.origin.y);
        self.size.contains(local)
    }

    pub fn max_exclusive(self) -> GridCoord2 {
        GridCoord2::new(
            self.origin
                .x
                .checked_add(usize_to_i32(self.size.width))
                .expect("grid rect max x overflows i32"),
            self.origin
                .y
                .checked_add(usize_to_i32(self.size.height))
                .expect("grid rect max y overflows i32"),
        )
    }

    pub fn iter(self) -> GridRectIter {
        GridRectIter {
            rect: self,
            next: self.origin,
            done: self.size.is_empty(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct GridRectIter {
    rect: GridRect,
    next: GridCoord2,
    done: bool,
}

impl Iterator for GridRectIter {
    type Item = GridCoord2;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }

        let current = self.next;
        let max = self.rect.max_exclusive();
        self.next.x += 1;

        if self.next.x >= max.x {
            self.next.x = self.rect.origin.x;
            self.next.y += 1;
        }

        if self.next.y >= max.y {
            self.done = true;
        }

        Some(current)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct GridBox {
    pub origin: GridCoord3,
    pub size: GridSize3,
}

impl GridBox {
    pub const fn new(origin: GridCoord3, size: GridSize3) -> Self {
        Self { origin, size }
    }

    pub const fn from_size(size: GridSize3) -> Self {
        Self::new(GridCoord3::ZERO, size)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum GridAxis {
    X,
    Y,
    Z,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum GridDirection2 {
    North,
    East,
    South,
    West,
}

impl GridDirection2 {
    pub const ALL: [Self; 4] = [Self::North, Self::East, Self::South, Self::West];

    pub const fn offset(self) -> GridCoord2 {
        match self {
            Self::North => GridCoord2::new(0, -1),
            Self::East => GridCoord2::new(1, 0),
            Self::South => GridCoord2::new(0, 1),
            Self::West => GridCoord2::new(-1, 0),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum GridDirection3 {
    North,
    East,
    South,
    West,
    Up,
    Down,
}

impl GridDirection3 {
    pub const ALL: [Self; 6] = [
        Self::North,
        Self::East,
        Self::South,
        Self::West,
        Self::Up,
        Self::Down,
    ];

    pub const fn offset(self) -> GridCoord3 {
        match self {
            Self::North => GridCoord3::new(0, -1, 0),
            Self::East => GridCoord3::new(1, 0, 0),
            Self::South => GridCoord3::new(0, 1, 0),
            Self::West => GridCoord3::new(-1, 0, 0),
            Self::Up => GridCoord3::new(0, 0, 1),
            Self::Down => GridCoord3::new(0, 0, -1),
        }
    }
}

pub const ORTHOGONAL_NEIGHBORS_2D: [GridCoord2; 4] = [
    GridCoord2::new(0, -1),
    GridCoord2::new(1, 0),
    GridCoord2::new(0, 1),
    GridCoord2::new(-1, 0),
];

pub const DIAGONAL_NEIGHBORS_2D: [GridCoord2; 4] = [
    GridCoord2::new(-1, -1),
    GridCoord2::new(1, -1),
    GridCoord2::new(-1, 1),
    GridCoord2::new(1, 1),
];

pub const ALL_NEIGHBORS_2D: [GridCoord2; 8] = [
    GridCoord2::new(-1, -1),
    GridCoord2::new(0, -1),
    GridCoord2::new(1, -1),
    GridCoord2::new(-1, 0),
    GridCoord2::new(1, 0),
    GridCoord2::new(-1, 1),
    GridCoord2::new(0, 1),
    GridCoord2::new(1, 1),
];

pub const DUAL_GRID_CORNER_OFFSETS: [GridCoord2; 4] = [
    GridCoord2::new(-1, -1),
    GridCoord2::new(0, -1),
    GridCoord2::new(-1, 0),
    GridCoord2::new(0, 0),
];

pub const AFFECTED_CORNER_OFFSETS: [GridCoord2; 4] = [
    GridCoord2::new(0, 0),
    GridCoord2::new(1, 0),
    GridCoord2::new(0, 1),
    GridCoord2::new(1, 1),
];

pub fn row_major_index(coord: GridCoord2, size: GridSize2) -> Option<usize> {
    if coord.x < 0 || coord.y < 0 {
        return None;
    }

    let x = coord.x as usize;
    let y = coord.y as usize;

    if x >= size.width || y >= size.height {
        return None;
    }

    Some(y * size.width + x)
}

pub fn coord_from_row_major_index(index: usize, size: GridSize2) -> Option<GridCoord2> {
    let area = size.checked_area()?;
    if size.width == 0 || index >= area {
        return None;
    }

    Some(GridCoord2::new(
        i32::try_from(index % size.width).ok()?,
        i32::try_from(index / size.width).ok()?,
    ))
}

pub fn row_major_index_3d(coord: GridCoord3, size: GridSize3) -> Option<usize> {
    if coord.x < 0 || coord.y < 0 || coord.z < 0 {
        return None;
    }

    let x = coord.x as usize;
    let y = coord.y as usize;
    let z = coord.z as usize;

    if x >= size.width || y >= size.height || z >= size.depth {
        return None;
    }

    Some((z * size.height + y) * size.width + x)
}

pub fn logic_cells_for_corner(corner: GridCoord2) -> [GridCoord2; 4] {
    DUAL_GRID_CORNER_OFFSETS.map(|offset| corner + offset)
}

pub fn affected_corners_for_logic_cell(cell: GridCoord2) -> [GridCoord2; 4] {
    AFFECTED_CORNER_OFFSETS.map(|offset| cell + offset)
}

fn usize_to_i32(value: usize) -> i32 {
    i32::try_from(value).expect("grid coordinate dimension exceeds i32::MAX")
}

#[cfg(feature = "glam")]
impl From<glam::IVec2> for GridCoord2 {
    fn from(value: glam::IVec2) -> Self {
        Self::new(value.x, value.y)
    }
}

#[cfg(feature = "glam")]
impl From<GridCoord2> for glam::IVec2 {
    fn from(value: GridCoord2) -> Self {
        Self::new(value.x, value.y)
    }
}

#[cfg(feature = "glam")]
impl From<glam::IVec3> for GridCoord3 {
    fn from(value: glam::IVec3) -> Self {
        Self::new(value.x, value.y, value.z)
    }
}

#[cfg(feature = "glam")]
impl From<GridCoord3> for glam::IVec3 {
    fn from(value: GridCoord3) -> Self {
        Self::new(value.x, value.y, value.z)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn row_major_index_rejects_negative_and_out_of_range_coords() {
        let size = GridSize2::new(3, 2);

        assert_eq!(row_major_index(GridCoord2::new(-1, 0), size), None);
        assert_eq!(row_major_index(GridCoord2::new(0, -1), size), None);
        assert_eq!(row_major_index(GridCoord2::new(3, 0), size), None);
        assert_eq!(row_major_index(GridCoord2::new(0, 2), size), None);
    }

    #[test]
    fn row_major_index_maps_valid_coords() {
        let size = GridSize2::new(3, 2);

        assert_eq!(row_major_index(GridCoord2::new(0, 0), size), Some(0));
        assert_eq!(row_major_index(GridCoord2::new(2, 0), size), Some(2));
        assert_eq!(row_major_index(GridCoord2::new(0, 1), size), Some(3));
        assert_eq!(row_major_index(GridCoord2::new(2, 1), size), Some(5));
        assert_eq!(
            coord_from_row_major_index(5, size),
            Some(GridCoord2::new(2, 1))
        );
    }

    #[test]
    fn grid_rect_iterates_exclusive_bounds() {
        let rect = GridRect::new(GridCoord2::new(1, 2), GridSize2::new(2, 2));
        let coords: Vec<_> = rect.iter().collect();

        assert_eq!(
            coords,
            vec![
                GridCoord2::new(1, 2),
                GridCoord2::new(2, 2),
                GridCoord2::new(1, 3),
                GridCoord2::new(2, 3),
            ]
        );
    }

    #[test]
    fn corner_and_dirty_helpers_preserve_contract() {
        let corner = GridCoord2::new(4, 7);

        assert_eq!(
            logic_cells_for_corner(corner),
            [
                GridCoord2::new(3, 6),
                GridCoord2::new(4, 6),
                GridCoord2::new(3, 7),
                GridCoord2::new(4, 7),
            ]
        );

        assert_eq!(
            affected_corners_for_logic_cell(GridCoord2::new(4, 7)),
            [
                GridCoord2::new(4, 7),
                GridCoord2::new(5, 7),
                GridCoord2::new(4, 8),
                GridCoord2::new(5, 8),
            ]
        );
    }

    #[test]
    fn row_major_index_3d_maps_depth_after_rows() {
        let size = GridSize3::new(3, 2, 2);

        assert_eq!(row_major_index_3d(GridCoord3::new(0, 0, 0), size), Some(0));
        assert_eq!(row_major_index_3d(GridCoord3::new(2, 1, 0), size), Some(5));
        assert_eq!(row_major_index_3d(GridCoord3::new(0, 0, 1), size), Some(6));
        assert_eq!(row_major_index_3d(GridCoord3::new(-1, 0, 0), size), None);
    }

    #[test]
    fn checked_size_helpers_reject_overflowing_dimensions() {
        assert_eq!(GridSize2::new(usize::MAX, 2).checked_area(), None);
        assert_eq!(GridSize3::new(usize::MAX, 2, 2).checked_volume(), None);
        let too_wide = usize::try_from(i32::MAX).unwrap() + 1;
        assert!(!GridSize2::new(too_wide, 1).fits_i32_coords());
    }
}
