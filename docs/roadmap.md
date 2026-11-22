# Grid — Current Roadmap

**Current direction (22 November 2026):** engine-neutral grid math, storage, dual-grid topology and optional procedural generation. The owner authorized removal of Godot and lab-owned generation in [#4](https://github.com/aschenrot/grid/issues/4). Independent sampled scalar noise and configurable CA are separately specified in [#2](https://github.com/aschenrot/grid/issues/2).

## Architecture

- `grid_math`: coordinates, shapes and checked indexing.
- `grid_storage`: generic grid storage and read/write traits.
- `tile_topology`: four-bit masks, visual descriptors, rotations and dirty-corner invalidation.
- `grid_prelude`: curated neutral exports, no procedural dependency.
- `grid_generation`: optional reusable procedural algorithms, with no engine ownership.
- `cli_dual_grid_demo`: downstream demonstration of the neutral APIs.
- Future engine and Riesel integration adapters belong outside Grid core.

## Delivery gates

| Work | Current state and acceptance |
| --- | --- |
| **#4: Breaking clean cutover** | [Draft PR #5](https://github.com/aschenrot/grid/pull/5) removes Godot adapter, Lab/chunk code and dependencies. Requires exact-head Rust tests, locked Cargo metadata, and reviewed migration notice. GitHub CI currently fails before test steps; not merged. |
| **#2: Neutral algorithm contract** | [Draft research PR #3](https://github.com/aschenrot/grid/pull/3) tests candidate finite sampled scalar noise, explicit threshold and configurable CA. API remains in specification; verify legacy exact output and accepted seed/precision/region/border/budget semantics. |
| **Production Grid operations** | After #2 acceptance, implement and test neutral scalar sampling, thresholding and configurable synchronous CA; preserve authoritative existing `tile_topology`. |
| **Riesel Grid Workbench** | Independent Cargo consumer, with typed node authoring/execution owned by [Riesel #175](https://github.com/aschenrot/riesel/issues/175) and [#187](https://github.com/aschenrot/riesel/issues/187). Grid must not duplicate Riesel graph semantics or require a renderer. |
| **Further extensibility** | Region, chunk, lazy fields, GPU/ECS and remote references only when additional user workloads justify these abstractions. |

## Preserved contracts

Coordinate origin is top-left, positive X right/Y down. Dual-grid masks use NW, NE, SW and SE bits, and all 16 masks/rotations remain authoritative in `tile_topology`. Out-of-bounds topology samples are empty, while the existing fixed CA reads out-of-bounds as occupied: do not conflate them.
The existing `fill_with_noise` and fixed `cellular_automata_step` remain backwards-compatible behavior until a separately accepted change. Editable dimensions require fallible, budgeted preflight before calling infallible dense-grid constructors.

## Validation for each integration candidate

Run `cargo fmt --all -- --check`, `cargo test --locked`, `cargo test --locked -p grid_generation`, `cargo test --locked -p tile_topology`, `cargo run --locked -p cli_dual_grid_demo -- --seed 123`, and `cargo metadata --locked --no-deps`. Save the exact tested SHA and logs. Static source comparison does not substitute for actual Cargo tests.

## Breaking change and history

Removed `godot_grid`, GDExtension, `Lab*` types, chunk/halo generation, and related APIs. External consumers must pin their old source revision or migrate their adapter independently; [migration guide](migrations/godot-lab-cutover.md). The earlier Godot/Bevy phased roadmap is retained in Git history only and is **not current instructions**.
