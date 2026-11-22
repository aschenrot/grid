# Grid #2 — scalar noise and configurable CA boundary investigation

**State:** research-only; source test candidate, no accepted public API, no production code changes, no implementation authorization. Originally compared against `aschenrot/grid` main `dd05a2c857c9d0410cb375dbebc108bdaa11dc5c` on 2026-11-22. The owner-authorized Godot/lab cutover [PR #5](https://github.com/aschenrot/grid/pull/5) has since been **merged** into main as `74afb5e4603aec7e5d6151157774fe718b43a264`. Owning specification: [Grid #2](https://github.com/aschenrot/grid/issues/2). Reference product: [Riesel #175](https://github.com/aschenrot/riesel/issues/175). Current foundational research: [Grid PR #1](https://github.com/aschenrot/grid/pull/1).

## Outcome and ownership

Enable neutral, reusable finite 2D scalar sampling and explicit scalar-to-occupancy conversion, plus configurable protected-write synchronous CA. Grid owns domain storage, generation algorithms and dual-grid topology. A Riesel Workbench may *present* these as `Noise2D`, `Threshold`, `CellularAutomata2D`, `DualGrid2D` nodes with typed ports; Grid should **not** expose Riesel node IDs, type registries, a general executor or editor schemas. RunenRender owns optional presentation, not canonical grid values. A procedural noise source is not inherently a grid capability or a new universal Riesel capability; using Grid as a finite-grid sampling adapter is a practical *library ownership* choice, not an exclusive owner of all possible noise operations.

## Source-backed facts

`grid_generation::fill_with_noise` currently owns `NoiseFillSettings { scale:f32, threshold:f32, seed:u32 }`. The exact fused compatibility chain is:

1. `FastNoiseLite::new()`, signed seed bits reinterpreted via `i32::from_ne_bytes(seed.to_ne_bytes())`.
2. `NoiseType::OpenSimplex2`, `FractalType::FBm`, 4 octaves, frequency equal to `scale`.
3. `domain_warp_2d(coord.x as f32, coord.y as f32)` then `get_noise_2d(warped_x, warped_y)`.
4. Normalization `(sample + 1.0)/2.0`, then `(value*1.3).clamp(0.0,1.0)`.
5. Strict `value > threshold` yields `CellType::Wall`, otherwise `Empty`.

The current `cellular_automata_step` counts eight neighboring walls, **treats out-of-range reads as occupied**, and writes `Wall` when count >=5, otherwise `Empty`, regardless of previous-center-cell state. It has no protected-write policy. **Topology is different:** `tile_topology` treats out-of-range logic cells as **empty** when forming dual-grid masks, and authoritative tests cover mask/kind/rotation. A global out-of-bounds policy would silently break one of these domains.

`DenseGrid2D::filled` asserts dimensions fit `i32` coordinates and allocates the entire grid via `vec!`; `from_fn` also asserts coordinate fit and collects the entire allocation. A node accepting editable values must preflight sizes, checked area, bytes, finite settings and resource budgets **before** calling these infallible constructors. Research-only `check_candidate_request` illustrates the gate; its precise accepted error/empty-grid policy is unresolved.

### Historical documentation/source contradiction: resolved by explicit breaking cutover

`docs/generation-boundary.md` says Godot lab generator/chunk behavior stays outside the crate, but the actual `grid_generation/src/lib.rs` exports `LabGenerationSettings`, `LabGeneratedChunk`, `generate_lab_chunk`, `build_owned_halo_formation_layers` and related functions. Importantly, **`adapters/godot_grid/src/lib.rs` imports and calls these APIs** to generate lab chunk payloads, topology layers and owned-halo formation batches. This is an actual in-repository consumer, not dead or hypothetical code. The document is **outdated or describes an earlier intended ownership state**; the current runtime/adapter is evidence of present dependencies.

**Resolved independently via accepted breaking scope:** [Grid implementation #4](https://github.com/aschenrot/grid/issues/4) and merged [PR #5](https://github.com/aschenrot/grid/pull/5) intentionally removed the Godot adapter and its lab APIs and documented downstream migration. The prior code and lab consumers remain in Git history, but are **not part of current Grid main**. Grid #2 must preserve only the *current neutral* noise/CA contracts and must not resurrect lab code or recreate Godot dependencies.

## Alternatives and migration costs

| Choice | Benefit | Risks / migration cost |
| --- | --- | --- |
| A. Default eager `DenseGrid2D<f32>` scalar sampler plus explicit threshold | Matches FastNoiseLite and existing grid storage, one finite output, simple direct `Noise2D` UX; legacy fused exact-cell replay can be tested | A bare grid does not carry seed, world-space region, algorithm/settings provenance; append metadata at stage/operation boundary |
| B. Small versioned sampled-field artifact containing `DenseGrid2D<f32>` | Can pin seed, coordinate region, sample precision/algorithm version and external result identity; supports reproducible editor inspection | More public API and metadata evolution/serialization burden. Must not push Riesel-specific node IDs into Grid |
| C. `NoiseField2D` lazy mathematical field plus separate `SampleField2D` | Good for composition, arbitrary regions, higher dimensionality, deferred sampling | More complicated basic UX; field evaluation, cross-thread/process transport, domain ownership and deterministic snapshots require additional contracts. Do **not** make this the sole basic API |
| D. `DenseGrid2D<f64>` default scalar samples | Matches some downstream scientific/analytical consumers | Existing FastNoiseLite API and algorithm are `f32`; converting after generation adds no source precision, changes memory/transport cost, and can introduce non-bit-exact JSON expectations. Need independently proven precision requirement |
| E. Put noise algorithm/CA inside Riesel core or one capability per node | Convenient first integration | Incorrect domain ownership and limits generality. Riesel should own graph semantics, not grid transforms or a compulsory provider process per node |

**Provisional research preference: A**, with B only when a real external consumer needs persistent artifact semantics and C as an independent advanced operation, not mandatory on the basic path. This is not accepted as an API choice. Retaining `fill_with_noise` compatibility is non-negotiable unless a separately versioned migration is approved.

For CA, compare (1) a single configurable synchronous `step` with distinct read-OOB and protected-write policy, leaving iterations/snapshots to the app, against (2) a bounded multi-step helper where the *domain* owns iterations. Prefer a small compatibility profile `birth_min=5, survive_min=5, outside=Occupied, protected_border=0` that can reproduce existing `cellular_automata_step`. A node can independently expose erosion-oriented `birth_min=5, survive_min=4, protected_border=1`. Rule extensibility must not require arbitrary closure serialization or second graph scheduler.

## Executable candidate and negative tests

`crates/grid_generation/tests/contract_2_scalar_ca.rs` uses **actual** `fastnoise_lite`, `grid_generation::fill_with_noise`, `grid_generation::cellular_automata_step`, `DenseGrid2D` and `tile_topology::form_visual_tiles`. All proposed APIs/rules and request validation in this file are **test-local**. The tests cover:

- Existing fused implementation versus separate actual sampled scalar + explicit strict threshold: exact cell-by-cell output for 4 seeds × 3 frequencies × 4 threshold settings at 64×64.
- Scalar reuse for a separate mean/heat-statistics consumer, deterministic replay and explicit strict-threshold boundary behavior.
- Legacy fixed CA versus candidate configurable CA with birth/survival=5 and OOB=occupied on 1×1, narrow, 2×2, 16×17 and 64×64 grids.
- Mixed protected border retained *at each step* over 5 CA iterations; final authoritative dual-grid dimensions are 65×65.
- 1×1 divergent OOB read policies and independent protected-write behavior.
- Candidate fallible rejection of zero size, out-of-coordinate-range dimensions, bytes exceeding a bounded budget, negative/nonfinite frequency and invalid threshold before invoking panic-capable grid constructors.

To reproduce from a clean Grid checkout at the exact PR head with Rust 1.100.0:

```sh
cargo test --locked -p grid_generation --test contract_2_scalar_ca -- --nocapture
cargo test --locked -p grid_generation --lib
cargo test --locked -p tile_topology
```

**Verified research CI (historical branch head):** [GitHub Actions run 45002130037, attempt 2](https://github.com/aschenrot/grid/actions/runs/45002130037), source head `ef5772f2cab5e8b0d8e46f87e433e58944916fec`, **SUCCESS** on GitHub-hosted Linux/Rust 1.100.0. The workflow actually executed and passed the exact scalar/CA candidate test binary, existing generation library tests and the authoritative topology tests. The earlier failed attempts were account-exhausted *private-repository* Actions usage, not observed Rust failures. The owner made Grid public and CI resumed. **Re-validate on an updated head containing the merged #4 clean cutover** before considering this research integrated or an API accepted.

## Decision gates and bounded follow-up

1. Rebase/merge accepted main `74afb5e4603aec7e5d6151157774fe718b43a264` non-destructively into the candidate and re-run exact-head Linux CI; retain logs for any failures.
2. Review any sample/threshold parity failures or invalid numeric policy choices; compare A/B/C/D and record maintainer acceptance/revision in Grid #2.
3. Verify the new branch inherits #4's current `docs/generation-boundary.md` and that current Grid remains Godot-free; no lab migration is in scope.
4. Only then open implementation issues for neutral Grid operations, their tests and independent Riesel Workbench integration, respecting the separate #187 typed-value execution specification.

No production API implementation, lab migration, renderer or Graph SDK is authorized by this research document.
