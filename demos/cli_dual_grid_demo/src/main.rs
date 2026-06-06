use grid_generation::{NoiseFillSettings, fill_with_noise};
use grid_prelude::*;

const WIDTH: usize = 16;
const HEIGHT: usize = 16;

fn main() {
    let seed = std::env::args()
        .nth(1)
        .and_then(|arg| arg.parse::<u32>().ok())
        .unwrap_or(42);

    let mut grid = DenseGrid2D::new(WIDTH, HEIGHT, CellType::Empty);
    fill_with_noise(&mut grid, NoiseFillSettings::new(0.1, 0.8, seed));

    println!("Seed: {seed}");
    println!("Logic grid:");
    print_logic_grid(&grid);

    let visual = form_visual_tiles(&grid);
    println!();
    println!("Visual dual grid:");
    print_visual_legend();
    print_visual_grid(&visual);
}

fn print_logic_grid(grid: &DenseGrid2D<CellType>) {
    for y in 0..grid.height() {
        for x in 0..grid.width() {
            let coord = GridCoord2::new(x as i32, y as i32);
            let symbol = match grid.get(coord) {
                Some(CellType::Wall) => '#',
                _ => '.',
            };
            print!("{symbol} ");
        }
        println!();
    }
}

fn print_visual_grid(grid: &DenseGrid2D<VisualTile>) {
    for y in 0..grid.height() {
        for x in 0..grid.width() {
            let coord = GridCoord2::new(x as i32, y as i32);
            let tile = grid.get(coord).copied().unwrap_or_default();
            print!("{tile} ");
        }
        println!();
    }
}

fn print_visual_legend() {
    println!("Legend: . empty, C corner, - edge, D diagonal, T t-junction, F full, ? debug");
}
