use std::fmt;

pub use grid_math::GridCoord2;
use grid_math::{GridSize2, affected_corners_for_logic_cell, logic_cells_for_corner};
pub use grid_storage::{DenseGrid2D, GridRead};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub enum CellType {
    #[default]
    Empty,
    Wall,
}

pub trait Occupancy {
    fn is_occupied(&self) -> bool;
}

impl Occupancy for CellType {
    fn is_occupied(&self) -> bool {
        matches!(self, Self::Wall)
    }
}

impl Occupancy for bool {
    fn is_occupied(&self) -> bool {
        *self
    }
}

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, Hash)]
pub struct TileMask4(u8);

pub type DualGridMask = TileMask4;

impl TileMask4 {
    pub const EMPTY: Self = Self(0);
    pub const FULL: Self = Self(0b1111);

    pub const fn from_bits(bits: u8) -> Option<Self> {
        if bits <= 0b1111 {
            Some(Self(bits))
        } else {
            None
        }
    }

    pub const fn from_bits_truncate(bits: u8) -> Self {
        Self(bits & 0b1111)
    }

    pub const fn bits(self) -> u8 {
        self.0
    }

    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }
}

impl From<TileMask4> for u8 {
    fn from(mask: TileMask4) -> Self {
        mask.bits()
    }
}

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct LogicCellCoord(GridCoord2);

impl LogicCellCoord {
    pub const ZERO: Self = Self(GridCoord2::ZERO);

    pub const fn new(coord: GridCoord2) -> Self {
        Self(coord)
    }

    pub const fn from_xy(x: i32, y: i32) -> Self {
        Self(GridCoord2::new(x, y))
    }

    pub const fn coord(self) -> GridCoord2 {
        self.0
    }

    pub const fn x(self) -> i32 {
        self.0.x
    }

    pub const fn y(self) -> i32 {
        self.0.y
    }

    pub const fn offset(self, dx: i32, dy: i32) -> Self {
        Self(self.0.offset(dx, dy))
    }
}

impl From<GridCoord2> for LogicCellCoord {
    fn from(coord: GridCoord2) -> Self {
        Self::new(coord)
    }
}

impl From<LogicCellCoord> for GridCoord2 {
    fn from(coord: LogicCellCoord) -> Self {
        coord.coord()
    }
}

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct VisualCornerCoord(GridCoord2);

impl VisualCornerCoord {
    pub const ZERO: Self = Self(GridCoord2::ZERO);

    pub const fn new(coord: GridCoord2) -> Self {
        Self(coord)
    }

    pub const fn from_xy(x: i32, y: i32) -> Self {
        Self(GridCoord2::new(x, y))
    }

    pub const fn coord(self) -> GridCoord2 {
        self.0
    }

    pub const fn x(self) -> i32 {
        self.0.x
    }

    pub const fn y(self) -> i32 {
        self.0.y
    }

    pub const fn offset(self, dx: i32, dy: i32) -> Self {
        Self(self.0.offset(dx, dy))
    }
}

impl From<GridCoord2> for VisualCornerCoord {
    fn from(coord: GridCoord2) -> Self {
        Self::new(coord)
    }
}

impl From<VisualCornerCoord> for GridCoord2 {
    fn from(coord: VisualCornerCoord) -> Self {
        coord.coord()
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct DualGridCorner {
    coord: VisualCornerCoord,
}

impl DualGridCorner {
    pub const fn from_visual_coord(coord: VisualCornerCoord) -> Self {
        Self { coord }
    }

    pub const fn visual_coord(self) -> VisualCornerCoord {
        self.coord
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Default)]
pub enum VisualTileKind {
    #[default]
    Empty,
    Corner,
    Edge,
    T,
    Diagonal,
    Full,
    Debug,
}

pub type TileClass = VisualTileKind;

impl fmt::Display for VisualTileKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let c = match self {
            Self::Empty => '.',
            Self::Corner => 'C',
            Self::Edge => '-',
            Self::T => 'T',
            Self::Diagonal => 'D',
            Self::Full => 'F',
            Self::Debug => '?',
        };
        write!(f, "{c}")
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Default)]
pub struct VisualTileRotation {
    degrees_cw: u16,
}

impl VisualTileRotation {
    pub const ZERO: Self = Self::new(0);
    pub const DEG_90: Self = Self::new(90);
    pub const DEG_180: Self = Self::new(180);
    pub const DEG_270: Self = Self::new(270);

    pub const fn new(degrees_cw: u16) -> Self {
        Self {
            degrees_cw: degrees_cw % 360,
        }
    }

    pub const fn degrees_cw(self) -> u16 {
        self.degrees_cw
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct VisualTileDescriptor {
    mask: TileMask4,
    kind: VisualTileKind,
    rotation: VisualTileRotation,
}

pub type VisualTileVariant = VisualTileDescriptor;

impl VisualTileDescriptor {
    pub const fn new(mask: TileMask4, kind: VisualTileKind, rotation: VisualTileRotation) -> Self {
        Self {
            mask,
            kind,
            rotation,
        }
    }

    pub fn from_mask(mask: TileMask4) -> Self {
        let kind = visual_tile_kind(mask);
        let rotation = visual_tile_rotation(mask, kind);

        Self::new(mask, kind, rotation)
    }

    pub fn from_mask_bits(bits: u8) -> Self {
        match TileMask4::from_bits(bits) {
            Some(mask) => Self::from_mask(mask),
            None => Self::new(
                TileMask4::from_bits_truncate(bits),
                VisualTileKind::Debug,
                VisualTileRotation::ZERO,
            ),
        }
    }

    pub const fn mask(self) -> TileMask4 {
        self.mask
    }

    pub const fn kind(self) -> VisualTileKind {
        self.kind
    }

    pub const fn rotation(self) -> VisualTileRotation {
        self.rotation
    }
}

impl Default for VisualTileDescriptor {
    fn default() -> Self {
        Self::from_mask(TileMask4::EMPTY)
    }
}

impl fmt::Display for VisualTileDescriptor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.kind)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct VisualTile {
    corner: DualGridCorner,
    descriptor: VisualTileDescriptor,
}

impl VisualTile {
    pub const fn new(corner: DualGridCorner, descriptor: VisualTileDescriptor) -> Self {
        Self { corner, descriptor }
    }

    pub const fn corner(self) -> DualGridCorner {
        self.corner
    }

    pub const fn visual_corner_coord(self) -> VisualCornerCoord {
        self.corner.visual_coord()
    }

    pub const fn descriptor(self) -> VisualTileDescriptor {
        self.descriptor
    }

    pub const fn is_occupied(self) -> bool {
        !self.descriptor.mask().is_empty()
    }
}

impl Default for VisualTile {
    fn default() -> Self {
        Self::new(
            DualGridCorner::from_visual_coord(VisualCornerCoord::ZERO),
            VisualTileDescriptor::default(),
        )
    }
}

impl fmt::Display for VisualTile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.descriptor)
    }
}

pub trait TileRule {
    fn visual_tile_for_mask(&self, corner: DualGridCorner, mask: TileMask4) -> VisualTile;
}

#[derive(Debug, Copy, Clone, Default)]
pub struct StandardTileRule;

impl TileRule for StandardTileRule {
    fn visual_tile_for_mask(&self, corner: DualGridCorner, mask: TileMask4) -> VisualTile {
        VisualTile::new(corner, VisualTileDescriptor::from_mask(mask))
    }
}

#[derive(Debug, Copy, Clone)]
pub struct DualGridResolver<R = StandardTileRule> {
    rule: R,
}

impl Default for DualGridResolver<StandardTileRule> {
    fn default() -> Self {
        Self {
            rule: StandardTileRule,
        }
    }
}

impl<R: TileRule> DualGridResolver<R> {
    pub const fn new(rule: R) -> Self {
        Self { rule }
    }

    pub fn form_visual_tiles<G>(&self, grid: &G) -> DenseGrid2D<VisualTile>
    where
        G: GridRead,
        G::Cell: Occupancy,
    {
        form_visual_tiles_with_rule(grid, &self.rule)
    }

    pub fn resolve_logic_cell<G>(&self, grid: &G, cell: LogicCellCoord) -> [VisualTile; 4]
    where
        G: GridRead,
        G::Cell: Occupancy,
    {
        affected_visual_corners_for_logic_cell(cell).map(|corner_coord| {
            let corner = DualGridCorner::from_visual_coord(corner_coord);
            let mask = dual_grid_mask_at(grid, corner_coord);
            self.rule.visual_tile_for_mask(corner, mask)
        })
    }
}

pub fn dual_grid_mask_at<G>(grid: &G, corner: VisualCornerCoord) -> TileMask4
where
    G: GridRead,
    G::Cell: Occupancy,
{
    let mut mask = 0u8;

    for (bit, coord) in logic_cells_for_visual_corner(corner)
        .into_iter()
        .enumerate()
    {
        if grid.get(coord.coord()).is_some_and(Occupancy::is_occupied) {
            mask |= 1 << bit;
        }
    }

    TileMask4::from_bits_truncate(mask)
}

pub fn dual_grid_mask_for_block_origin<G>(grid: &G, origin: LogicCellCoord) -> TileMask4
where
    G: GridRead,
    G::Cell: Occupancy,
{
    dual_grid_mask_at(grid, VisualCornerCoord::new(origin.coord().offset(1, 1)))
}

pub fn form_visual_tiles<G>(grid: &G) -> DenseGrid2D<VisualTile>
where
    G: GridRead,
    G::Cell: Occupancy,
{
    form_visual_tiles_with_rule(grid, &StandardTileRule)
}

pub fn form_visual_tiles_with_rule<G, R>(grid: &G, rule: &R) -> DenseGrid2D<VisualTile>
where
    G: GridRead,
    G::Cell: Occupancy,
    R: TileRule,
{
    let logic_size = grid.size();
    let visual_size = GridSize2::new(logic_size.width + 1, logic_size.height + 1);

    DenseGrid2D::from_fn(visual_size, |coord| {
        let visual_corner = VisualCornerCoord::new(coord);
        let corner = DualGridCorner::from_visual_coord(visual_corner);
        let mask = dual_grid_mask_at(grid, visual_corner);
        rule.visual_tile_for_mask(corner, mask)
    })
}

pub fn affected_visual_corners_for_logic_cell(cell: LogicCellCoord) -> [VisualCornerCoord; 4] {
    affected_corners_for_logic_cell(cell.coord()).map(VisualCornerCoord::new)
}

pub fn logic_cells_for_visual_corner(corner: VisualCornerCoord) -> [LogicCellCoord; 4] {
    logic_cells_for_corner(corner.coord()).map(LogicCellCoord::new)
}

pub fn visual_tile_kind(mask: TileMask4) -> VisualTileKind {
    match mask.bits() {
        0b0000 => VisualTileKind::Empty,
        0b0001 | 0b0010 | 0b0100 | 0b1000 => VisualTileKind::Corner,
        0b0011 | 0b0101 | 0b1010 | 0b1100 => VisualTileKind::Edge,
        0b0110 | 0b1001 => VisualTileKind::Diagonal,
        0b0111 | 0b1011 | 0b1101 | 0b1110 => VisualTileKind::T,
        0b1111 => VisualTileKind::Full,
        _ => VisualTileKind::Debug,
    }
}

pub fn visual_tile_rotation(mask: TileMask4, kind: VisualTileKind) -> VisualTileRotation {
    let degrees = match kind {
        VisualTileKind::Corner => match mask.bits() {
            0b0001 => 0,
            0b0010 => 90,
            0b0100 => 180,
            0b1000 => 270,
            _ => 0,
        },
        VisualTileKind::Edge => match mask.bits() {
            0b0011 => 0,
            0b0101 => 90,
            0b1010 => 270,
            0b1100 => 180,
            _ => 0,
        },
        VisualTileKind::Diagonal => match mask.bits() {
            0b1001 => 0,
            0b0110 => 90,
            _ => 0,
        },
        VisualTileKind::T => match mask.bits() {
            0b1110 => 0,
            0b0111 => 90,
            0b1011 => 180,
            0b1101 => 270,
            _ => 0,
        },
        _ => 0,
    };

    VisualTileRotation::new(degrees)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn grid_from_walls(width: usize, height: usize, walls: &[GridCoord2]) -> DenseGrid2D<CellType> {
        let mut grid = DenseGrid2D::new(width, height, CellType::Empty);
        for &wall in walls {
            assert!(grid.set(wall, CellType::Wall));
        }
        grid
    }

    #[test]
    fn classifies_all_16_masks_with_rotation() {
        let expected = [
            (0b0000, VisualTileKind::Empty, 0),
            (0b0001, VisualTileKind::Corner, 0),
            (0b0010, VisualTileKind::Corner, 90),
            (0b0011, VisualTileKind::Edge, 0),
            (0b0100, VisualTileKind::Corner, 180),
            (0b0101, VisualTileKind::Edge, 90),
            (0b0110, VisualTileKind::Diagonal, 90),
            (0b0111, VisualTileKind::T, 90),
            (0b1000, VisualTileKind::Corner, 270),
            (0b1001, VisualTileKind::Diagonal, 0),
            (0b1010, VisualTileKind::Edge, 270),
            (0b1011, VisualTileKind::T, 180),
            (0b1100, VisualTileKind::Edge, 180),
            (0b1101, VisualTileKind::T, 270),
            (0b1110, VisualTileKind::T, 0),
            (0b1111, VisualTileKind::Full, 0),
        ];

        for (bits, kind, rotation) in expected {
            let descriptor = VisualTileDescriptor::from_mask_bits(bits);

            assert_eq!(descriptor.mask().bits(), bits);
            assert_eq!(descriptor.kind(), kind, "mask {bits:04b}");
            assert_eq!(
                descriptor.rotation().degrees_cw(),
                rotation,
                "mask {bits:04b}"
            );
        }
    }

    #[test]
    fn computes_masks_for_all_corner_bit_positions() {
        let grid = grid_from_walls(
            2,
            2,
            &[
                GridCoord2::new(0, 0),
                GridCoord2::new(1, 0),
                GridCoord2::new(0, 1),
                GridCoord2::new(1, 1),
            ],
        );

        assert_eq!(
            dual_grid_mask_at(&grid, VisualCornerCoord::from_xy(1, 1)).bits(),
            0b1111
        );
    }

    #[test]
    fn visual_grid_is_one_larger_than_logic_grid() {
        let grid = DenseGrid2D::new(4, 3, CellType::Empty);
        let visual = form_visual_tiles(&grid);

        assert_eq!(visual.size(), GridSize2::new(5, 4));
    }

    #[test]
    fn negative_and_out_of_range_neighbors_are_empty() {
        let grid = grid_from_walls(1, 1, &[GridCoord2::new(0, 0)]);

        assert_eq!(
            dual_grid_mask_at(&grid, VisualCornerCoord::from_xy(0, 0)).bits(),
            0b1000
        );
        assert_eq!(
            dual_grid_mask_at(&grid, VisualCornerCoord::from_xy(1, 0)).bits(),
            0b0100
        );
        assert_eq!(
            dual_grid_mask_at(&grid, VisualCornerCoord::from_xy(0, 1)).bits(),
            0b0010
        );
        assert_eq!(
            dual_grid_mask_at(&grid, VisualCornerCoord::from_xy(1, 1)).bits(),
            0b0001
        );
        assert_eq!(
            dual_grid_mask_at(&grid, VisualCornerCoord::from_xy(-1, -1)).bits(),
            0b0000
        );
    }

    #[test]
    fn changed_logic_cell_affects_exactly_four_visual_corners() {
        let affected = affected_visual_corners_for_logic_cell(LogicCellCoord::from_xy(3, 5));

        assert_eq!(
            affected,
            [
                VisualCornerCoord::from_xy(3, 5),
                VisualCornerCoord::from_xy(4, 5),
                VisualCornerCoord::from_xy(3, 6),
                VisualCornerCoord::from_xy(4, 6),
            ]
        );
    }

    #[test]
    fn typed_coords_make_corner_and_block_origin_masks_explicit() {
        let grid = grid_from_walls(2, 2, &[GridCoord2::new(0, 0)]);

        assert_eq!(
            dual_grid_mask_at(&grid, VisualCornerCoord::from_xy(0, 0)).bits(),
            0b1000
        );
        assert_eq!(
            dual_grid_mask_at(&grid, VisualCornerCoord::from_xy(1, 1)).bits(),
            0b0001
        );
        assert_eq!(
            dual_grid_mask_for_block_origin(&grid, LogicCellCoord::from_xy(0, 0)).bits(),
            0b0001
        );
    }

    #[test]
    fn typed_dirty_cell_invalidation_returns_visual_corner_coords() {
        let affected = affected_visual_corners_for_logic_cell(LogicCellCoord::from_xy(3, 5));

        assert_eq!(
            affected,
            [
                VisualCornerCoord::from_xy(3, 5),
                VisualCornerCoord::from_xy(4, 5),
                VisualCornerCoord::from_xy(3, 6),
                VisualCornerCoord::from_xy(4, 6),
            ]
        );
    }

    #[test]
    fn resolver_returns_descriptors_for_dirty_cell_corners() {
        let grid = grid_from_walls(2, 2, &[GridCoord2::new(1, 1)]);
        let resolver = DualGridResolver::default();
        let resolved = resolver.resolve_logic_cell(&grid, LogicCellCoord::from_xy(1, 1));
        let masks: Vec<_> = resolved
            .into_iter()
            .map(|tile| tile.descriptor().mask().bits())
            .collect();

        assert_eq!(masks, vec![0b1000, 0b0100, 0b0010, 0b0001]);
    }
}
