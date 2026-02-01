    // root/demo/src/main.rs
    use gridcore::*;

    fn main() {
        const DIMENSION: usize = 16;
        const WIDTH: usize = DIMENSION;
        const HEIGHT: usize = DIMENSION;

        let mut grid = Grid2d::<CellType>::new(WIDTH, HEIGHT);

        let seed: u32 = rand::random();
        println!("Using random seed: {}", seed);

        grid.fill_with_noise(0.1, 0.8, seed);

        // 4️⃣ Print logic grid
        println!("Logic grid:");
        // todo grid.print_logic(&grid);

        // 5️⃣ Generate mesh tiles (with automatic rotation)
        let mesh_grid = grid.mesh_tiles();

        // 6️⃣ Print visual grid
        println!("\nVisual mesh grid:");
        grid.print_visual(&mesh_grid);
    }
