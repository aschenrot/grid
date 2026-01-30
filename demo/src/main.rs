// src/main.rs

use glam::IVec2;
use gridcore::{
    DenseGrid2D, FastResolver, VisualResolver, fill_with_perlin, grid::{CellType, LogicCell, MeshPlacement, utils::print_mesh_grid}, print_grid_generic, resolver::MyRule
};

fn main() {
    println!("Hello, world!");

    // 1️⃣ Create a small 8x8 grid of LogicCells
    let width = 16;
    let height = 16;
    let default_cell = LogicCell { cell_type: CellType::Empty };
    let mut grid = DenseGrid2D::new(width, height, default_cell);

    // 2️⃣ Fill grid with Perlin noise (Floor vs Empty)
    fill_with_perlin(&mut grid, 10.0, |v| {
        if v > 0.0 {
            LogicCell { cell_type: CellType::Floor }
        } else {
            LogicCell { cell_type: CellType::Empty }
        }
    });

    // 3️⃣ Print the logic cell grid
    println!("\nLogicCell Grid:");
    print_grid_generic(&grid);

    // 4️⃣ Create a visual resolver using your custom rule
    let resolver = FastResolver::<MyRule, IVec2, LogicCell, _> {
        rule: MyRule,
        _phantom: Default::default(),
    };

    // 5️⃣ Resolve all mesh placements
    let placements: Vec<MeshPlacement> = resolver.resolve_all(&grid);

    // 6️⃣ Print the mesh placement grid
    println!("\nMesh Placement Grid:");
    print_mesh_grid(&grid, &placements);
}
