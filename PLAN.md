# SNAPHU: C to Rust Translation Plan

The c2rust dump in `src/snaphu_full.rs`, as well as the `snaphu-sys` Rust crate (that provides FFI bindings to the C code), proves the C code can compile, but the end-goal is to hand-write a safe, idiomatic Rust implementation that mirrors SNAPHU's behaviour while isolating state and depending on the published `cost_scaling_rs` crate instead of the vendored CS2 sources. This document lists the constraints, outlines the target Rust architecture, and defines an incremental translation workflow that follows the dependency order captured in `snaphu_translation_order.csv`. The bundled `complex_writer/` generator is incorporated as the very first milestone so we can run it via `cargo run --example complex_writer`, visualize wrapped interferograms, and capture authoritative fixtures that feed the translation effort.

## Guiding Principles
- **Single source of truth for state**: Replace global variables with explicit structs (e.g. `SnaphuContext`, `UnwrapConfig`, `TileState`). Pass immutable borrows where possible; constrain mutability to focused subsystems.
- **Prefer data structs over macros**: Translate C macros into enum/struct methods, helper functions, and consts. Use Rust traits when behaviour depends on type families (e.g. cost calculators or CLI option parsing).
- **Incrementally confirm parity**: Keep the C implementation runnable for cross-checking. For each translated slice expose tests/fixtures that compare C vs Rust outputs (golden files, property checks, small raster fixtures).
- **Leverage crates**: Depend on `cost_scaling_rs` for min-cost max-flow. Prefer slices + iterators to keep footprint low. Avoid extra dependencies

## Target Crate Layout
```
src/
  lib.rs                # Public API + CLI entry (gradually replaces c2rust binding)
  cli.rs                # arg parsing -> config builder (clap)
  context.rs            # aggregate of shared runtime state and buffers
  config/
    mod.rs              # structs for options, file paths, thresholds
    defaults.rs         # translation of default macros into consts
  data/
    raster.rs           # phase/coherence rasters, typed wrappers around Vec<f32>
    tile.rs             # per-tile metadata + assembly helpers
  costs/
    mod.rs              # traits for cost terms, smoothing/defo/topo calculators
    lookup.rs           # DZ/Rho lookup-table generation utilities
  network/
    mod.rs              # graph construction + wrappers around cost_scaling_rs
  unwrapping/
    lpn.rs              # LP-based cost calculators (CalcCostLP*)
    tiles.rs            # tile assembly & connected components (AssembleTile*)
    flow.rs             # Choose between MCF vs tree unwrappers
  io/
    reader.rs           # image reading, parameter files
    writer.rs           # unwrap results
examples/
  complex_writer.rs     # SNAPHU fixture generator + visualization hook (cargo run --example complex_writer)
```
Each module owns its portion of `SnaphuContext`; modules expose small structs with clearly documented inputs/outputs so that translation of one module can be validated before touching others.

## Managing Former Global State
1. **Inventory globals**: Extract from `snaphu_original/snaphu_full.c` using `rg "extern"` and function signatures. Categorise into (a) configuration parameters, (b) per-run mutable data, (c) temporary work buffers.
2. **Define state buckets**:
   - `RunConfig`: read-only values derived from CLI + config files.
   - `RuntimeState`: mutable data that spans phases (arrays, histograms, graph caches).
   - `TileWorkspace`: short-lived buffers used by tiling/connected-components.
3. **Constructor pattern**: Provide `Snaphu::new(config)` that initialises `RuntimeState` lazily as modules request them; this keeps memory usage predictable and allows injecting mocks for tests.
4. **Threading**: Plan for future parallelism by keeping state Send/Sync-friendly (Arc-split contexts, `rayon` under feature flag) though the initial port can stay single-threaded.

## Using `cost_scaling_rs`
- Add to `Cargo.toml`: `cost_scaling_rs = "0.1.0"`.
- Wrap the crate behind `network::MinCostFlowSolver` trait so we can stub it when validating early modules.
- Translate SNAPHU's `InitCs2`, `SolveMCF`, etc. by mapping their inputs to the solver’s struct-based API.
- Keep CS2-specific config (`MAXARCS`, scaling tolerances) expressed as Rust consts managed by the network module.

## Translation Workflow (driven by `snaphu_translation_order.csv`)
0. **Prepare the `complex_writer` example**
   - Move `complex_writer/` into the main workspace (or port its logic into `examples/complex_writer.rs`) so it builds with `cargo run --example complex_writer` and picks up shared dependencies.
   - Remove the hard-coded `SNAPHU_PATH`; instead, accept a CLI flag (defaulting to `snaphu` on `$PATH` or the Rust binary under development) and document how to feed its outputs back into SNAPHU.
   - Keep the IO + visualization utilities intact and add instructions for saving wrapped/unwrapped/magnitude rasters so they can seed the golden-fixture suite later in this plan.
1. **Project scaffolding**
   - Move the c2rust translation behind a feature flag (`legacy-cli`) so the new crate can compile during the rewrite. Make sure the code is disabled because (src/snaphu_full.rs) has a LOT of lines of code, which slows downs compile times.
   - Create placeholders for the modules listed above with minimal structs and TODOs.
2. **Leaf utilities (Priority ≥5)**
   - Follow CSV entries to flesh out pure helpers: arithmetic kernels (`Add2DFloatArrays`, `BoxCarAvg`), lookup-table builders (`BuildDZRCritLookupTable`, `BuildDZRhoMaxLookupTable`), data structure helpers (`BucketInsert/Remove`, `AvgSigSq`).
   - Write unit tests that mirror known equations, enabling quick confirmation without the entire pipeline.
3. **Cost builders (Priority 4)**
   - Implement `BuildStatCosts*` in `costs` module, ensuring they only depend on previously translated helpers and `RuntimeState` slices.
   - Provide `CostField` structs to hold per-pixel arrays. Tests should read fixture rasters and verify deterministic bytes vs C output (use small 5×5 samples captured from the original binary).
4. **Tile assembly & graph prep (Priority 3)**
   - Translate `AssembleTiles`, `AssembleTileConnComps`, `BuildCostArrays`, `BuildCostArraysNonGrid`. These functions orchestrate multiple helpers, so confirm APIs for rasters/tiles are stable before moving on.
   - Introduce `TileGraph` struct encapsulating adjacency + tile metadata; expose conversions to `cost_scaling_rs` edges.
5. **Network + flow (Priority 2)**
   - Implement wrappers around `cost_scaling_rs` that build the full problem from `TileGraph` and the cost arrays.
   - Port `CalcFlow` and related functions, ensuring error handling maps into Rust `Result` types.
6. **Entry points (Priority 0–1)**
   - Translate `CalcCostLP*`, `CalcCostNonGrid`, CLI parsing, IO, and the high-level `main` pipeline.
   - Replace the call to `snaphu_sys::run_main` in `lib.rs` with the new `Snaphu::run()` once the rest compiles.

While translating each CSV-priority batch, update the spreadsheet (or a markdown checklist) with statuses so we know which functions remain.

## Handling Macros & Constants
- Convert flag-style macros (`#define USE_NEW_GRAPH 1`) into `const bool` or enums inside the config module.
- Replace inline macro math (e.g. `#define SQR(x) ((x)*(x))`) with `fn sqr(x: f32) -> f32` or `num::Float::powi` where clarity matters. These functions will be inlined
- Multi-line macros that encapsulate logic should become private helper functions with well-typed parameters, making ownership explicit.

## Testing & Validation Strategy
- **Golden fixtures**: Capture small interferogram tiles + expected unwrap outputs from the C binary. Re-run after each major module to prevent drift.
- **Property tests**: For cost calculators and lookup tables, assert monotonicity, ranges, and invariants mirrored from SNAPHU docs.
- **Integration harness**: Keep `snaphu_full.rs` accessible via `cargo test --features legacy-cli` to run the original pipeline as a reference until parity is achieved.
- **Example-driven datasets**: Use `cargo run --example complex_writer` to regenerate wrapped interferograms on demand, collect the resulting `.bin` / `.out` files, and compare SNAPHU vs SNAPHU-rs outputs to refresh fixtures whenever upstream logic evolves.

## Risk & Mitigation
- **Global-state coupling**: Some functions rely on implicit writes; when translating, add temporary logging to the C version to trace which globals each function touches.
- **Floating-point drift**: Document acceptable tolerances per module; prefer `f64` for accumulators even if C used double.
- **CS2 parity**: `cost_scaling_rs` may expose different tuning knobs; add adapter layer to emulate SNAPHU defaults and keep integration tests focused on network outputs instead of internal solver steps.

## Definition of Done
1. `cargo test` passes with only the idiomatic Rust implementation (no legacy feature flag required).
2. CLI arguments, config file semantics, and output formats match the original binary (validated via regression suite).
3. All functions listed in `snaphu_translation_order.csv` marked `Status=done` with references to their Rust counterparts.
