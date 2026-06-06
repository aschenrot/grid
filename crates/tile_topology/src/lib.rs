use std::fmt;

pub use grid_math::GridCoord2;
use grid_math::{GridSize2, affected_corners_for_logic_cell, logic_cells_for_corner};
pub use grid_storage::{DenseGrid2D, GridRead};

/// Minimal built-in cell type used by examples and demos.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub enum CellType {
    #[default]
    Empty,
    Wall,
}

/// Reports whether a logic-grid cell participates in topology masks.
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

/// Four-bit dual-grid occupancy mask for one visual corner.
///
/// For visual corner `(cx, cy)`, bit order is:
///
/// - bit 0: logic cell `(cx - 1, cy - 1)`
/// - bit 1: logic cell `(cx, cy - 1)`
/// - bit 2: logic cell `(cx - 1, cy)`
/// - bit 3: logic cell `(cx, cy)`
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

/// Coordinate in the logic-cell grid.
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

/// Coordinate in the visual corner grid.
///
/// A visual corner `(cx, cy)` samples the four logic cells around that corner.
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

/// One visual-grid corner with explicit coordinate semantics.
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

/// Neutral visual topology class for a 4-bit mask.
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

/// Clockwise rotation for a neutral visual tile descriptor.
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

/// Neutral classification of a visual corner.
///
/// Engine adapters map this descriptor to assets/resources. It deliberately
/// does not contain mesh ids, transforms, nodes, handles, or ECS data.
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

/// Visual-grid tile data for one visual corner.
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

/// Policy hook for converting a mask into a visual tile.
pub trait TileRule {
    fn visual_tile_for_mask(&self, corner: DualGridCorner, mask: TileMask4) -> VisualTile;
}

/// Standard mask-to-kind and mask-to-rotation rule.
#[derive(Debug, Copy, Clone, Default)]
pub struct StandardTileRule;

impl TileRule for StandardTileRule {
    fn visual_tile_for_mask(&self, corner: DualGridCorner, mask: TileMask4) -> VisualTile {
        VisualTile::new(corner, VisualTileDescriptor::from_mask(mask))
    }
}

/// Resolver that computes visual tiles from an occupancy grid.
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

/// Computes the 4-bit mask at a visual corner.
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

/// Computes a mask from a 2x2 logic-cell block origin.
///
/// This is a compatibility helper for prototype code that names the sample by
/// its top-left logic cell. `origin` maps to visual corner `(origin.x + 1,
/// origin.y + 1)`.
pub fn dual_grid_mask_for_block_origin<G>(grid: &G, origin: LogicCellCoord) -> TileMask4
where
    G: GridRead,
    G::Cell: Occupancy,
{
    dual_grid_mask_at(grid, VisualCornerCoord::new(origin.coord().offset(1, 1)))
}

/// Forms a `(width + 1) x (height + 1)` visual grid from a logic grid.
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

/// Returns exactly the four visual corners affected by one changed logic cell.
pub fn affected_visual_corners_for_logic_cell(cell: LogicCellCoord) -> [VisualCornerCoord; 4] {
    affected_corners_for_logic_cell(cell.coord()).map(VisualCornerCoord::new)
}

/// Returns the four logic cells sampled by one visual corner in mask bit order.
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

    const SINGLE_WALL_FIXTURE: &str =
        include_str!("../../../tests/fixtures/dual_grid_single_wall.txt");

    fn grid_from_walls(width: usize, height: usize, walls: &[GridCoord2]) -> DenseGrid2D<CellType> {
        let mut grid = DenseGrid2D::new(width, height, CellType::Empty);
        for &wall in walls {
            assert!(grid.set(wall, CellType::Wall));
        }
        grid
    }

    fn fixture_section<'a>(fixture: &'a str, section: &str) -> Vec<&'a str> {
        let heading = format!("[{section}]");
        let mut in_section = false;
        let mut lines = Vec::new();

        for line in fixture.lines().map(str::trim) {
            if line.starts_with('[') && line.ends_with(']') {
                in_section = line == heading;
                continue;
            }

            if in_section && !line.is_empty() {
                lines.push(line);
            }
        }

        lines
    }

    fn grid_from_logic_fixture(lines: &[&str]) -> DenseGrid2D<CellType> {
        let height = lines.len();
        let width = lines.first().map_or(0, |line| line.chars().count());

        DenseGrid2D::from_fn(GridSize2::new(width, height), |coord| {
            let y = usize::try_from(coord.y).expect("fixture y must be non-negative");
            let x = usize::try_from(coord.x).expect("fixture x must be non-negative");
            match lines[y].as_bytes()[x] {
                b'#' => CellType::Wall,
                b'.' => CellType::Empty,
                other => panic!("unsupported fixture logic cell byte {other}"),
            }
        })
    }

    fn coord_from_fixture_line(line: &str) -> GridCoord2 {
        let mut parts = line.split_whitespace();
        let x = parts
            .next()
            .expect("fixture coord x")
            .parse()
            .expect("fixture coord x must be i32");
        let y = parts
            .next()
            .expect("fixture coord y")
            .parse()
            .expect("fixture coord y must be i32");
        assert!(parts.next().is_none(), "fixture coord has extra values");

        GridCoord2::new(x, y)
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

    #[test]
    fn fixture_single_wall_preserves_visual_corner_contract() {
        let logic_lines = fixture_section(SINGLE_WALL_FIXTURE, "logic");
        let grid = grid_from_logic_fixture(&logic_lines);
        let visual = form_visual_tiles(&grid);

        let expected_masks: Vec<Vec<u8>> = fixture_section(SINGLE_WALL_FIXTURE, "visual_masks")
            .into_iter()
            .map(|line| {
                line.split_whitespace()
                    .map(|bits| u8::from_str_radix(bits, 2).expect("fixture mask must be binary"))
                    .collect()
            })
            .collect();

        for (y, row) in expected_masks.iter().enumerate() {
            for (x, expected) in row.iter().enumerate() {
                let coord = GridCoord2::new(
                    i32::try_from(x).expect("fixture x must fit i32"),
                    i32::try_from(y).expect("fixture y must fit i32"),
                );
                let actual = visual
                    .get(coord)
                    .expect("fixture visual coord must exist")
                    .descriptor()
                    .mask()
                    .bits();

                assert_eq!(actual, *expected, "visual mask at {coord:?}");
            }
        }

        let affected_cell = LogicCellCoord::new(coord_from_fixture_line(
            fixture_section(SINGLE_WALL_FIXTURE, "affected_cell")[0],
        ));
        let affected: Vec<_> = affected_visual_corners_for_logic_cell(affected_cell)
            .into_iter()
            .map(VisualCornerCoord::coord)
            .collect();
        let expected_affected: Vec<_> = fixture_section(SINGLE_WALL_FIXTURE, "affected_corners")
            .into_iter()
            .map(coord_from_fixture_line)
            .collect();

        assert_eq!(affected, expected_affected);
    }
}
