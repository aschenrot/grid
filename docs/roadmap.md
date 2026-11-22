# Roadmap

Grid's current scope is a reusable Rust grid library, independent of game-engine runtime and graph-authoring code.

## Delivered

- Grid math, storage, dual-grid topology, optional procedural helpers and CLI demonstration.
- [Godot/lab cleanup #4](https://github.com/aschenrot/grid/issues/4), merged in [PR #5](https://github.com/aschenrot/grid/pull/5). Previous adapter APIs are covered by the [migration guide](migrations/godot-lab-cutover.md).

## Open

- [#2 — sampled scalar noise and configurable CA](https://github.com/aschenrot/grid/issues/2): decide numerical, coordinate-region, iteration, boundary and resource semantics; validate the contract before adding public APIs. [Research PR #3](https://github.com/aschenrot/grid/pull/3) contains test-local candidates.
- [#9 — API safety and documentation audit](https://github.com/aschenrot/grid/issues/9): clarify extreme coordinates, correct current docs, and review CI/asset policy.
- [#7 — public release review](https://github.com/aschenrot/grid/issues/7): publication rights, license files and attribution.

These issues are the source of live status; this page describes delivery order, not transient CI results.

## Next delivery boundary

After accepting #2, implement and test engine-neutral sampled scalar values, explicit thresholding, and configurable synchronous CA while preserving the current `fill_with_noise` and fixed CA behavior where compatibility is required. Keep topology mask/rotation semantics in `tile_topology`.

Riesel node composition and execution are downstream application work, tracked separately in [Riesel #175](https://github.com/aschenrot/riesel/issues/175) and [#187](https://github.com/aschenrot/riesel/issues/187). No Grid graph runtime is planned from this reference use case alone.

## Validation

```sh
cargo fmt --all -- --check
cargo test --locked
cargo test --locked -p grid_generation
cargo test --locked -p tile_topology
cargo run --locked -p cli_dual_grid_demo -- --seed 123
```

The current Linux workflow runs these commands. Broader workspace/feature/edge-case CI coverage is being reviewed in #9.
