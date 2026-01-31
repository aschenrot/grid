// root/demo/src/main.rs
use gridcore::*;

fn main() {
    use CellType::{Wall as W, Empty as E};
    // 3️⃣ Create a 3x3 logic grid using the macro
    let grid = grid![
        [W]
    ];

    // 4️⃣ Print logic grid
    println!("Logic grid:");
    grid.print_logic(|c| match c {
        CellType::Wall => 'W',
        CellType::Empty => 'E',
    });

    // 5️⃣ Precompute mask → corner type lookup
    let mask_to_corner = [
        CornerType::Empty,    // 0b0000
        CornerType::Corner,   // 0b0001
        CornerType::Corner,   // 0b0010
        CornerType::Edge,     // 0b0011
        CornerType::Corner,   // 0b0100
        CornerType::Edge,     // 0b0101
        CornerType::Diagonal, // 0b0110
        CornerType::T,        // 0b0111
        CornerType::Corner,   // 0b1000
        CornerType::Diagonal, // 0b1001
        CornerType::Edge,     // 0b1010
        CornerType::T,        // 0b1011
        CornerType::Edge,     // 0b1100
        CornerType::T,        // 0b1101
        CornerType::T,        // 0b1110
        CornerType::Full,     // 0b1111
    ];

    // 6️⃣ Generate visual corner grid using the LUT
    let visual_grid = grid.corner_tiles(|mask, _logic| mask_to_corner[mask as usize]);

    // 7️⃣ Print visual grid
    println!("\nVisual grid:");
    grid.print_visual(&visual_grid, |c| match c.visual {
        CornerType::Empty => 'E',
        CornerType::Corner => 'C',
        CornerType::Edge => '─',
        CornerType::Diagonal => 'D',
        CornerType::T => 'T',
        CornerType::Full => 'F',
        CornerType::Debug => '?',
    });
}
