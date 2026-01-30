use fastnoise_lite::{FastNoiseLite, NoiseType};
use glam::IVec2;
use rand::{rng, Rng};
use crate::{GridWrite, GridBounds};

pub fn fill_with_perlin<G, T, F>(grid: &mut G, scale: f32, map_fn: F)
where
	G: GridWrite<Coord = IVec2, Cell = T> + GridBounds<Coord = IVec2>,
	F: Fn(f32) -> T,
{
	let mut rng = rng();
	let seed: i32 = rng.random(); // random seed

	let mut noise = FastNoiseLite::new();
	noise.set_seed(Option::from(seed));
	noise.set_noise_type(Some(NoiseType::Perlin));

	for y in grid.min().y..=grid.max().y {
		for x in grid.min().x..=grid.max().x {
			let v = noise.get_noise_2d(x as f32 * scale, y as f32 * scale);
			let cell = map_fn(v);
			grid.set(IVec2::new(x, y), cell);
		}
	}
}
