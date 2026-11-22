# Generation Boundary

`grid_generation` is an optional consumer of math, storage and topology. Core crates do not depend on procedural generation.

## Current APIs

- `NoiseFillSettings` and `SeededGridGenerator` provide seed and noise settings.
- `fill_with_noise` samples FastNoiseLite noise and applies a strict threshold, writing `CellType` into a dense 2D grid.
- `cellular_automata_step` uses a fixed synchronous eight-neighbor rule and treats out-of-bounds reads as occupied.
- `test_pattern` produces demonstration occupancy data.

These existing functions are the compatibility baseline. Separately sampled scalar values, explicit thresholding and configurable CA are under specification in [Grid #2](https://github.com/aschenrot/grid/issues/2).

## Ownership

Generation returns grid-domain data, not renderer objects, execution graphs or application-owned world state. The previous Godot-specific chunk and halo generator was removed in [PR #5](https://github.com/aschenrot/grid/pull/5); see the [migration guide](migrations/godot-lab-cutover.md).
