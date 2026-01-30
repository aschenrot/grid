use std::fmt;
use glam::IVec2;
use gridcore::grid::{MeshId, MeshPlacement};
use gridcore::resolver::resolver::CoordOps;
use gridcore::{DenseGrid2D, FastResolver, GridBounds, GridRead, VisualResolver, fill_with_perlin, print_grid_generic};
use gridcore::resolver::TileRule;

// -----------------------------
// 1. Define your logic cell
// -----------------------------
#[derive(Clone, Copy, Debug, PartialEq)]
enum Tile {
    Empty,
    Wall,
    Floor,
}

impl fmt::Display for Tile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Tile::Empty => write!(f, "*"),
            Tile::Wall  => write!(f, "#"),
            Tile::Floor => write!(f, "."),
        }
    }
}

// -----------------------------
// 2. Wall variants
// -----------------------------
#[derive(Clone, Copy)]
enum WallVariant {
    Single, Horizontal, Vertical,
    CornerTL, CornerTR, CornerBL, CornerBR,
    TUp, TDown, TLeft, TRight,
    Cross,
}

// -----------------------------
// 3. TileRule
// -----------------------------
struct WallAutotileRule;

impl TileRule<IVec2, Tile> for WallAutotileRule {
    fn compute(&self, corner: IVec2, neighbors: [Option<&Tile>; 4]) -> Option<MeshPlacement> {
        let center = neighbors[3]?; // bottom-right of the 2x2 corner

        let mesh_id = match center {
            Tile::Wall => {
                // simple wall autotiling based on neighbors
                let tl = neighbors[0].copied().unwrap_or(Tile::Empty);
                let tr = neighbors[1].copied().unwrap_or(Tile::Empty);
                let bl = neighbors[2].copied().unwrap_or(Tile::Empty);

                let variant = match (tl == Tile::Wall, tr == Tile::Wall, bl == Tile::Wall) {
                    (false, false, false) => WallVariant::Single,
                    (true, true, true)    => WallVariant::Cross,
                    (true, false, false)  => WallVariant::CornerTL,
                    (false, true, false)  => WallVariant::CornerTR,
                    (false, false, true)  => WallVariant::CornerBL,
                    (true, true, false)   => WallVariant::TDown,
                    (true, false, true)   => WallVariant::TLeft,
                    (false, true, true)   => WallVariant::TRight,
                    _                     => WallVariant::Horizontal,
                };

                MeshId(match variant {
                    WallVariant::Single => 01,
                    WallVariant::Horizontal => 02,
                    WallVariant::Vertical => 03,
                    WallVariant::CornerTL => 04,
                    WallVariant::CornerTR => 05,
                    WallVariant::CornerBL => 06,
                    WallVariant::CornerBR => 07,
                    WallVariant::TUp => 08,
                    WallVariant::TDown => 09,
                    WallVariant::TLeft => 10,
                    WallVariant::TRight => 11,
                    WallVariant::Cross => 12,
                })
            }
            Tile::Floor => MeshId(20),
            Tile::Empty => MeshId(30),
        };

        Some(MeshPlacement {
            mesh_id,
            position: glam::Vec3::new(corner.x as f32, corner.y as f32, 0.0),
            rotation: glam::Quat::IDENTITY,
            mirror_x: false,
            mirror_y: false,
            metadata: 0,
        })
    }
}

// -----------------------------
// 4. Mesh Grid Printer
// -----------------------------

pub fn print_mesh_grid<C, G>(grid: &G, placements: &[MeshPlacement])
where
    G: GridRead<Coord = C> + GridBounds<Coord = C>,
    C: CoordOps,
{
    let min = grid.min();
    let max = grid.max();

    for y in min.y()..=max.y() {
        for x in min.x()..=max.x() {
            let id = placements
                .iter()
                .find(|p| p.position.x as i32 == x && p.position.y as i32 == y)
                .map(|p| match p.mesh_id.0 {
                    1..=12 => format!("{:02}", p.mesh_id.0), // walls
                    20 => "F.".to_string(),                  // floor
                    30 => "E.".to_string(),                  // empty
                    _ => "..".to_string(),
                })
                .unwrap_or("..".to_string());

            print!("{} ", id);
        }
        println!();
    }
}

// -----------------------------
// 5. Main
// -----------------------------
fn main() {
    let width = 16;
    let height = 16;

    let mut grid = DenseGrid2D::new(width, height, Tile::Empty);

    fill_with_perlin(&mut grid, 10.0, |v| {
        if v < -0.2 { Tile::Wall }
        else if v < 0.2 { Tile::Floor }
        else { Tile::Empty }
    });

    println!("Generated Grid:");
    print_grid_generic(&grid);

    let resolver = FastResolver::<_, IVec2, Tile, _> {
        rule: WallAutotileRule,
        _phantom: std::marker::PhantomData,
    };

    let placements = resolver.resolve_all(&grid);

    println!("Generated Mesh Grid:");
    print_mesh_grid::<IVec2, _>(&grid, &placements);
}
