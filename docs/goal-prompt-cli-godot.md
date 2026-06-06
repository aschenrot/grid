# Goal Prompt: Land Core Split, CLI Demo, and Godot Adapter

Use this prompt to drive an implementation goal for `/Users/joshua/Projekte/grid`.

```text
Complete the Crystonix/grid roadmap through the point where the core crates,
CLI dual-grid demo, and Godot adapter have landed.

Repository:
/Users/joshua/Projekte/grid

Related prototype inputs:
- /Users/joshua/Projekte/grid/gridcore
- /Users/joshua/Projekte/gridcore

Authoritative roadmap:
- /Users/joshua/Projekte/grid/docs/roadmap.md

Hard constraints:
- Read and follow applicable repository instructions before planning or editing.
- Treat /Users/joshua/Projekte/grid as the source of truth.
- Treat /Users/joshua/Projekte/gridcore as prototype input only.
- Do not change Runenwerk in this goal.
- Do not implement the Bevy adapter in this goal.
- Do not revert unrelated user changes.
- Keep core crates engine-neutral.
- Core crates must not depend on Godot, Bevy, Runenwerk, ECS, renderer APIs,
  asset systems, fastnoise-lite, rand, or required glam.
- `glam` conversions may exist only behind an optional feature.
- Godot-specific code must live in `adapters/godot_grid`.
- Godot adapter code must map neutral topology descriptors to Godot-facing
  resources/nodes/data; it must not own mask computation, tile classification,
  dirty-cell invalidation, or topology truth.

Target final scope for this goal:
- `crates/grid_math`
- `crates/grid_storage`
- `crates/tile_topology`
- `crates/grid_generation`
- `crates/grid_prelude`
- `demos/cli_dual_grid_demo`
- `adapters/godot_grid`
- docs needed to explain architecture, boundaries, dual-grid behavior, adapter
  contracts, and migration.

Out of scope:
- Bevy adapter implementation.
- Runenwerk integration implementation.
- Publishing crates.
- Large feature additions beyond the existing dual-grid/autotiling prototype.
- Rewriting the project into a different architecture than the accepted
  roadmap.

Execution plan:

1. Inspect current repo state.
   - Check `git status --short`.
   - Read `docs/roadmap.md`.
   - Read current `/Users/joshua/Projekte/grid/gridcore` APIs.
   - Read relevant files from `/Users/joshua/Projekte/gridcore` for the abstract
     resolver prototype.
   - Identify unrelated dirty files and avoid touching them.

2. Complete Phase 0: design freeze.
   - Add or update:
     - `docs/architecture.md`
     - `docs/crate-boundaries.md`
     - `docs/dual-grid-model.md`
     - `docs/adapter-contracts.md`
     - migration notes for both prototypes.
   - Freeze:
     - coordinate origin
     - corner coordinate semantics
     - 4-bit mask bit order
     - rotation convention
     - out-of-bounds behavior
     - dirty-cell to affected-corners invalidation
   - Do not start file moves until these contracts are explicit.

3. Complete Phase 1: characterization tests.
   - Add tests for all 16 masks.
   - Add tests for visual tile kind and rotation.
   - Add tests for `width + 1` by `height + 1` visual grid dimensions.
   - Add tests for negative and out-of-range coordinates.
   - Add tests proving a changed logic cell affects exactly four visual corners.
   - Run `cargo test -p gridcore`.

4. Complete Phase 2: extract `grid_math`.
   - Create `crates/grid_math`.
   - Add owned coordinate, size, rect/box, direction, bounds, neighbor, and index
     helpers.
   - Remove required `glam` from core APIs.
   - Add optional `glam` conversions behind a feature if useful.
   - Run:
     - `cargo test -p grid_math`
     - `cargo test`

5. Complete Phase 3: extract `grid_storage`.
   - Create `crates/grid_storage`.
   - Move dense grid storage and access traits here.
   - Merge useful `GridRead`, `GridWrite`, and `GridBounds` ideas from the
     separate `/Users/joshua/Projekte/gridcore` prototype.
   - Keep sparse/chunked storage minimal unless needed for the demo or tests.
   - Ensure `grid_storage` depends only on `grid_math`.
   - Run:
     - `cargo test -p grid_storage`
     - `cargo test`

6. Complete Phase 4: extract `tile_topology`.
   - Create `crates/tile_topology`.
   - Move occupancy, dual-grid mask formation, visual tile classification,
     rotation, descriptor creation, and affected-corner invalidation here.
   - Rename renderer-facing concepts:
     - `MeshType` -> `VisualTileKind`
     - `MeshId` -> `VisualTileDescriptor` or `VisualTileVariant`
     - `mesh_tiles()` -> `form_visual_tiles()`
   - Remove core `MeshPlacement`, `Vec3`, and `Quat` concepts.
   - Ensure `tile_topology` depends only on `grid_math` and `grid_storage`.
   - Run:
     - `cargo test -p tile_topology`
     - `cargo test`

7. Complete Phase 5: extract `grid_generation`.
   - Create `crates/grid_generation`.
   - Move noise/random generation here.
   - Keep `fastnoise-lite` and `rand` out of core crates.
   - Add deterministic seed tests.
   - Run:
     - `cargo test -p grid_generation`
     - `cargo test --workspace --default-members`

8. Complete Phase 6: prelude and CLI demo.
   - Create `crates/grid_prelude`.
   - Move `demo` to `demos/cli_dual_grid_demo`.
   - Make the CLI demo use public APIs only.
   - Update `README.md` with normal usage.
   - Ensure default workspace members include only core crates.
   - Run:
     - `cargo test --workspace --default-members`
     - `cargo run -p cli_dual_grid_demo`

9. Complete Godot adapter contract and implementation.
   - Use `docs/adapter-contracts.md` as the contract.
   - Create `adapters/godot_grid`.
   - Keep it outside `default-members`.
   - Implement thin Godot-facing mapping around neutral core descriptors.
   - Do not duplicate mask or classification logic.
   - Keep Godot asset/node/resource concepts adapter-local.
   - Add tests or compile checks that validate mapping behavior without needing
     a full Godot editor run if possible.
   - Run:
     - `cargo test --workspace --default-members`
     - `cargo test -p godot_grid`

10. Final validation and cleanup.
    - Run the smallest relevant checks after each phase.
    - At the end, run:
      - `cargo test --workspace --default-members`
      - `cargo run -p cli_dual_grid_demo`
      - `cargo test -p godot_grid`
    - Run `cargo fmt` if Rust files changed.
    - Check `git status --short`.
    - Report all changed files, exact modules/functions touched, why they
      changed, and validation results.

Stop conditions:
- Stop and report clearly if the Godot dependency cannot compile on the local
  machine.
- Stop and report clearly if the adapter requires a binding/API decision that
  cannot be inferred from the existing code or docs.
- Do not proceed into Runenwerk implementation.
- Do not proceed into Bevy adapter implementation.

Completion definition:
- `/Users/joshua/Projekte/grid` has the split core workspace.
- Default checks build only core crates.
- Core crates are engine-neutral and dependency-light.
- CLI demo has landed under `demos/cli_dual_grid_demo`.
- Godot adapter has landed under `adapters/godot_grid`.
- Prototype behavior from both grid prototypes is migrated, renamed, or
  explicitly discarded in docs.
- Tests cover storage, bounds, all 16 masks, rotation, visual-grid dimensions,
  and affected-corner invalidation.
- Validation commands listed above pass, or any failure is documented with the
  exact blocker and next action.
```
