# Grid Workspace Roadmap

This roadmap makes `/Users/joshua/Projekte/grid` the source of truth for the
reusable grid and dual-grid topology work. The separate
`/Users/joshua/Projekte/gridcore` repository is treated as a prototype input,
not as a second long-term implementation.

## Target Shape

```text
Crystonix/grid
|-- Cargo.toml
|-- README.md
|-- docs/
|   |-- roadmap.md
|   |-- architecture.md
|   |-- crate-boundaries.md
|   |-- dual-grid-model.md
|   |-- adapter-contracts.md
|   `-- integration-runenwerk-godot-bevy.md
|-- crates/
|   |-- grid_math/
|   |-- grid_storage/
|   |-- tile_topology/
|   |-- grid_generation/
|   `-- grid_prelude/
|-- adapters/
|   |-- godot_grid/
|   `-- bevy_grid/
|-- demos/
|   |-- cli_dual_grid_demo/
|   |-- godot_dual_grid_demo/
|   `-- bevy_dual_grid_demo/
`-- tests/
    `-- fixtures/
```

### Workspace Policy

Core crates are the default build surface:

```toml
[workspace]
resolver = "3"
members = [
  "crates/grid_math",
  "crates/grid_storage",
  "crates/tile_topology",
  "crates/grid_generation",
  "crates/grid_prelude",
  "adapters/godot_grid",
  "adapters/bevy_grid",
  "demos/cli_dual_grid_demo",
  "demos/godot_dual_grid_demo",
  "demos/bevy_dual_grid_demo",
]

default-members = [
  "crates/grid_math",
  "crates/grid_storage",
  "crates/tile_topology",
  "crates/grid_prelude",
]
```

Normal validation should not compile Godot, Bevy, or demo dependencies unless
the task explicitly touches them.

## Dependency Direction

```text
grid_math
  |
  v
grid_storage
  |
  v
tile_topology
  |
  v
grid_prelude

grid_generation -> grid_math + grid_storage + tile_topology

godot_grid -> grid_math + grid_storage + tile_topology + godot
bevy_grid  -> grid_math + grid_storage + tile_topology + bevy
cli demo   -> grid_prelude + optional grid_generation
```

Rules:

- Core crates do not depend on Godot, Bevy, Runenwerk, ECS, renderer APIs, or
  asset systems.
- `grid_generation` consumes core crates. Core crates do not consume generation.
- Adapters map neutral core descriptors to engine resources. They do not own
  dual-grid topology truth.
- Runenwerk integration is a downstream consumer task, not a dependency of this
  repository.

## Naming Contract

Renderer-facing names should leave core topology:

| Current concept | Final core name |
| --- | --- |
| `MeshType` | `VisualTileKind` |
| `MeshId` | `VisualTileDescriptor` or `VisualTileVariant` |
| `MeshPlacement` | adapter-side placement, not core topology |
| `mesh_tiles()` | `form_visual_tiles()` |
| `Grid2d` | `DenseGrid2D` or `DenseGrid2D<T>` |
| `corner_mask()` | `dual_grid_mask_at()` |

## Phase 0: Design Freeze

Goal: freeze the public contract before moving files.

Tasks:

- `G0.1` Write `docs/architecture.md`.
- `G0.2` Write `docs/crate-boundaries.md`.
- `G0.3` Write `docs/dual-grid-model.md`.
- `G0.4` Write migration notes for current `/Users/joshua/Projekte/grid`.
- `G0.5` Write migration notes for `/Users/joshua/Projekte/gridcore`.

Acceptance:

- The docs define coordinate origin, corner coordinate semantics, mask bit order,
  rotation convention, out-of-bounds behavior, and dirty-cell invalidation.
- The docs state which prototype APIs are kept, renamed, moved, or discarded.
- No crate split starts until this phase is accepted.

Validation:

```text
cargo test
```

## Phase 1: Characterization Tests

Goal: lock the current behavior before refactoring.

Tasks:

- `G1.1` Add tests for all 16 dual-grid masks.
- `G1.2` Add tests for `MeshId::from_mask` classification and rotation.
- `G1.3` Add tests proving visual grid dimensions are `width + 1` by
  `height + 1`.
- `G1.4` Add tests for negative and out-of-range coordinates.
- `G1.5` Add tests proving a changed logic cell affects exactly four visual
  corners.

Acceptance:

- Existing behavior is covered even where naming is still temporary.
- Bugs found by tests are either fixed immediately or documented as intended
  behavior changes for Phase 4.

Validation:

```text
cargo test -p gridcore
```

## Phase 2: Extract `grid_math`

Goal: create a dependency-light coordinate and bounds crate.

Target public surface:

```text
GridCoord2
GridCoord3
GridSize2
GridSize3
GridRect
GridBox
GridDirection2
GridDirection3
GridAxis
neighbor offsets
corner/cell coordinate helpers
row-major index helpers
inclusive/exclusive bounds iteration
```

Tasks:

- `G2.1` Create `crates/grid_math`.
- `G2.2` Move coordinate/index/bounds logic out of current `Grid2d`.
- `G2.3` Replace hard `glam::IVec2` use in core APIs with owned coordinate
  types.
- `G2.4` Add optional `glam` conversions behind a `glam` feature.
- `G2.5` Add unit tests for index math, bounds, and negative coordinates.

Acceptance:

- `grid_math` has no required dependencies.
- `grid_math` has no tile, generation, renderer, ECS, or engine concepts.
- Existing demo behavior still works through compatibility shims or updated
  call sites.

Validation:

```text
cargo test -p grid_math
cargo test
```

## Phase 3: Extract `grid_storage`

Goal: separate reusable containers and access traits from topology.

Target public surface:

```text
GridRead
GridWrite
GridBounds
DenseGrid2D
DenseGrid3D
SparseGrid2D
SparseGrid3D
ChunkedGrid2D
```

Tasks:

- `G3.1` Create `crates/grid_storage`.
- `G3.2` Move `DenseGrid2D`/`Grid2d` storage logic into `grid_storage`.
- `G3.3` Merge useful access traits from `/Users/joshua/Projekte/gridcore`.
- `G3.4` Keep sparse/chunked storage as documented placeholders unless needed
  by tests or demo.
- `G3.5` Add iterator and indexing tests.

Acceptance:

- `grid_storage` depends only on `grid_math`.
- `grid_storage` does not know about occupancy, masks, visual tiles, generation,
  Godot, Bevy, or Runenwerk.
- Dense storage can back the same dual-grid demo as before.

Validation:

```text
cargo test -p grid_storage
cargo test
```

## Phase 4: Extract `tile_topology`

Goal: make dual-grid/autotiling the explicit topology crate.

Target public surface:

```text
Occupancy
TileClass
TileMask4
DualGridMask
DualGridCorner
VisualTile
VisualTileKind
VisualTileRotation
VisualTileVariant
VisualTileDescriptor
DualGridResolver
TileRule
affected_corners_for_cell()
dual_grid_mask_at()
form_visual_tiles()
```

Tasks:

- `G4.1` Create `crates/tile_topology`.
- `G4.2` Move `Occupancy`, mask formation, tile classification, and visual tile
  formation into `tile_topology`.
- `G4.3` Rename renderer-facing concepts to neutral visual topology concepts.
- `G4.4` Merge the separate `gridcore` resolver ideas only where they improve
  the final API.
- `G4.5` Remove core `MeshPlacement`, `Vec3`, and `Quat` concepts.
- `G4.6` Add tests for mask, classification, rotation, and invalidation.

Acceptance:

- `tile_topology` depends only on `grid_math` and `grid_storage`.
- All 16 masks have explicit expected output.
- `form_visual_tiles()` returns neutral descriptors, not engine assets.
- Dirty-cell invalidation is deterministic and documented.

Validation:

```text
cargo test -p tile_topology
cargo test
```

## Phase 5: Extract `grid_generation`

Goal: keep procedural helpers optional and outside core.

Target public surface:

```text
NoiseFillSettings
SeededGridGenerator
fill_with_noise()
cellular_automata_step()
test_pattern()
```

Tasks:

- `G5.1` Create `crates/grid_generation`.
- `G5.2` Move `fill_with_noise` out of core topology/storage.
- `G5.3` Keep `fastnoise-lite` and `rand` dependencies here only.
- `G5.4` Add deterministic seed tests.

Acceptance:

- No core crate depends on `fastnoise-lite` or `rand`.
- Generation works as a consumer of `grid_storage` and `tile_topology`.
- The CLI demo can opt into generation.

Validation:

```text
cargo test -p grid_generation
cargo test --workspace --default-members
```

## Phase 6: Prelude, Demo, and Public Docs

Goal: make normal usage discoverable.

Tasks:

- `G6.1` Create `crates/grid_prelude`.
- `G6.2` Move `demo` to `demos/cli_dual_grid_demo`.
- `G6.3` Update `README.md` with the normal import path and a minimal example.
- `G6.4` Add a usage guide that is separate from architecture docs.
- `G6.5` Add doc examples for common flows.

Acceptance:

- `grid_prelude` is only curated re-exports.
- The CLI demo uses public APIs only.
- README and demo agree on the preferred public usage style.

Validation:

```text
cargo test --workspace --default-members
cargo run -p cli_dual_grid_demo
```

## Phase 7: Adapter Contracts

Goal: define adapters before implementing engine-specific crates.

Tasks:

- `G7.1` Write `docs/adapter-contracts.md`.
- `G7.2` Define how `VisualTileDescriptor` maps to an engine asset key.
- `G7.3` Define how adapter placement handles position, rotation, mirroring,
  variant selection, and missing assets.
- `G7.4` Decide whether Godot and Bevy adapters live in this repo immediately
  or stay as future crates.

Acceptance:

- The adapter contract is engine-neutral.
- Topology remains in `tile_topology`.
- Asset loading, ECS entities, nodes, scenes, bundles, and mesh handles remain
  adapter concerns.

Validation:

```text
cargo test --workspace --default-members
```

## Phase 8: Optional Godot and Bevy Adapters

Goal: add adapters only after the core API is stable.

Tasks:

- `G8.1` Create `adapters/godot_grid` if Godot integration is needed.
- `G8.2` Create `adapters/bevy_grid` if Bevy integration is needed.
- `G8.3` Keep adapters outside default workspace members.
- `G8.4` Add smoke examples for adapter mapping.

Acceptance:

- Adapters compile independently.
- Core default checks remain fast.
- No topology logic is duplicated in adapters.

Validation:

```text
cargo test --workspace --default-members
cargo test -p godot_grid
cargo test -p bevy_grid
```

## Phase 9: Runenwerk Integration Plan

Goal: integrate as a downstream consumer without coupling this repo to
Runenwerk.

Tasks:

- `G9.1` Write `docs/integration-runenwerk-godot-bevy.md`.
- `G9.2` Map `grid_math` coordinates to Runenwerk `domain/spatial` types.
- `G9.3` Map `VisualTileDescriptor` to Runenwerk product/assets contracts.
- `G9.4` In Runenwerk, run its architecture governance routine before code
  changes.
- `G9.5` Add a thin Runenwerk-owned adapter or domain integration crate there,
  not here, unless a reusable adapter contract emerges.

Acceptance:

- Runenwerk consumes grid crates through a clear boundary.
- This repo does not depend on Runenwerk.
- Runenwerk's existing `domain/spatial`, `domain/chunking`,
  `domain/procgen`, and `domain/world_sdf` ownership is respected.

Validation in this repo:

```text
cargo test --workspace --default-members
```

Validation in Runenwerk:

```text
task ai:architecture-governance -- --task "Integrate Crystonix/grid core crates" --scope "Runenwerk downstream grid adapter"
```

Use Runenwerk's generated validation checklist after that command.

## Completion Definition

The roadmap is complete when:

- Core crates are split and default checks build only core.
- Core public APIs do not require `glam`, `rand`, `fastnoise-lite`, Godot,
  Bevy, Runenwerk, ECS, renderer APIs, or asset systems.
- All current prototype behavior is either migrated, deliberately renamed, or
  explicitly discarded.
- The CLI demo uses public APIs.
- Tests cover masks, rotations, storage, bounds, and dirty-cell invalidation.
- Adapter and Runenwerk integration contracts are documented before adapter
  implementation.

## Current Blockers

- The current repo and the separate `gridcore` repo overlap. This roadmap makes
  the current repo authoritative.
- The mask coordinate contract is not yet frozen.
- The current code has no real behavioral tests.
- `glam`, `rand`, and `fastnoise-lite` are currently required by the monolithic
  `gridcore` crate.
- Renderer-facing names are embedded in core APIs.
- Runenwerk has its own spatial/chunking/procgen/world-sdf contracts and must be
  treated as a downstream integration, not as an upstream design dependency.

## Recommended Execution Order

Execute exactly in phase order through Phase 6 before starting adapters or
Runenwerk integration:

```text
Phase 0 -> Phase 1 -> Phase 2 -> Phase 3 -> Phase 4 -> Phase 5 -> Phase 6
```

Phases 7 through 9 should begin only after the core API survives at least one
demo and one downstream API review.
