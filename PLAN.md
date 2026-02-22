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

The CSV file `snaphu_translation_order.csv` contains a list of all the functions in the original SNAPHU C source code, topologically sorted by the call graph ordering. The "Priority" column in the CSV goes from 0 to 9, where the top-level functions in the call graph are 0, while the leaf nodes have priority 9.

You can use the translation plan script to easily lookup the CSV: `just translation_plan --help`.

The slash command `/translate_function [function_name]` will help translate a SNAPHU C function to Rust.

0. **Prepare the `complex_writer` example** *(DONE)*
   - `complex_writer` now lives under `examples/complex_writer.rs` with a clap-powered CLI (`--width/--height`, `--wraps`, `--snaphu-bin`, `--snaphu-arg`, `--skip-snaphu`, `--rerun`, `--write-png`). Running `cargo run --example complex_writer -- …` emits `wrapped_phase.bin`, `snaphu.out`, and (optionally) a PNG preview inside `target/examples/complex_writer/`.
   - The tool accepts a configurable SNAPHU binary path instead of relying on a hard-coded absolute path, and forwards extra arguments so we can exercise different C features while gathering fixtures.
   - Rerun logging, PNG dumps, and README instructions are in place, so the example doubles as a visualization harness and a reproducible dataset generator for parity tests.
1. **Project scaffolding** *(DONE)*
   - The c2rust translation now sits behind the `legacy-cli` feature gate and the crate compiles quickly with `--no-default-features`, giving us a clean slate for the idiomatic rewrite.
   - Placeholder modules (`cli`, `config`, `context`, `data`, `costs`, `network`, `unwrapping`, `io`) exist with minimal structs so the new architecture can be filled in incrementally.
2. **Leaf utilities (Priority ≥5)** *(IN PROGRESS)*
   - ✅ Arithmetic kernels now live under `data::ops` with unit coverage for `Add2DFloatArrays`, `BoxCarAvg`, `AvgSigSq`, and `LClip`.
   - ✅ Lookup-table builders `BuildDZRCritLookupTable` and `BuildDZRhoMaxLookupTable` are implemented in `costs::lookup` with typed parameter structs and smoke tests.
   - ✅ Bucket helpers mirroring `BucketInsert/Remove` exist in `network::bucket` with invariants enforced by Rust errors/tests.
   - ✅ Mirror padding (`MirrorPad`) and interpolation helpers (`LinInterp1D/2D`) are available under `data::ops` with regression tests.
   - ✅ Phase wrapping utilities (`FlattenWrappedPhase`, `WrapPhase`, `ModDiff`) now live in `data::ops`, keeping wrapped values inside `[0, 2π)` and matching the short-cycle behaviour from the C code.
   - ✅ Wrapped-gradient calculators (`CalcWrappedRangeDiffs/AzDiffs`) have idiomatic equivalents in `costs::gradients`, exposing typed outputs for cost builders and reusing the shared boxcar/mirror-pad helpers.
   - ✅ Masking helpers (`MaskCost`, `MaskSmoothCost`, `MaskPrespecifiedArcCosts`) are available in `costs::types` along with strongly typed cost records, so future modules can drop the pointer-heavy c2rust shims.
   - ✅ Mirror padding now has a `mirror_pad_with_fill` variant that works for any `Copy` type, so non-f32 rasters (e.g. shorts, custom structs) can reuse the same primitive.
   - ✅ Miscellaneous Level ≥5 math helpers (`LRound`, `LMin`) now live alongside `l_clip` so the rest of the cost code can stop calling into the legacy module.
   - ✅ Array sanity helpers (`ValidDataArray`, `NonNegDataArray`) have idiomatic equivalents in `data::ops`, keeping the raster validators in one place.
   - ✅ Network neighbor-loop helper `GetArcNumLims` now has a typed Rust equivalent in `network::mod` with explicit boundary-count validation and branch-coverage tests.
   - ✅ Output-file opener `OpenOutputFile` now lives in `io::writer` with `/tmp` fallback semantics, real-path reporting, and regression tests for primary/fallback/error paths.
   - ✅ Generic raster loader `Read2DArray` now exists in `io::reader` as a typed native-endian tile reader (`TileWindow` + `Raster<T>`), with bounds/size checks and file-layout regression tests.
   - ✅ Alternating-line raster loader `ReadAltLineFile` now lives in `io::reader`, returning typed magnitude/phase tiles with the original seek/stride layout and validation rules.
   - ✅ Phase-only alternating-line reader `ReadAltLineFilePhase` is implemented in `io::reader`, preserving the C phase-offset and row-stride semantics with typed `Raster<f32>` output.
   - ✅ Alternating-sample reader `ReadAltSampFile` now lives in `io::reader`, splitting interleaved `A/B` float samples into typed tile rasters with parity tests and strict size checks.
   - ✅ Packed arc reader `Read2DRowColFile` is available in `io::reader` as `RowColTile<T>` extraction, preserving row/column-block offsets and tile slicing rules from the C implementation.
   - ✅ Tree frontier helper `AddNewNode` now lives in `network::mod`, with explicit bucket-window semantics (`minind/maxind/curr`) and regression tests for reinsert, underflow, overflow, and predecessor-forced updates.
   - ✅ Flow/residue helpers (`CalcFlow`, `NodeResidue`) are now implemented in `data::ops` with row/column-layout parity tests.
   - ✅ EI/intensity helpers (`RemoveMean`, `SolveEIModelParams`) are now translated with typed Rust APIs (`data::ops` + `costs::lookup`) and validation coverage.
   - ✅ IO wrappers (`ReadIntensity`, `ReadCorrelation`, `Write2DArray`, `WriteAltLineFile`, `WriteAltSampFile`) now exist as safe typed readers/writers in `io::{reader,writer}`.
   - ✅ Additional graph helpers (`ClosestNode`, `RegionsNeighborNode`, `ScanRegion`, `CheckLeaf`, `CheckBoundary`) are now available in `network::mod` with explicit traversal/consistency tests.
   - ✅ `TraceSecondaryArc` now has a typed translation in `unwrapping::tiles`, split into cost-profile tracing and secondary-graph registration helpers, with convergence/zero-cost/reuse path tests.
   - ✅ Config parsing helpers `StringToDouble`/`StringToLong` now live in `config::mod`, preserving SNAPHU's full-string parse checks, infinity/overflow guards, and legacy empty-string edge behavior.
   - Remaining work in this phase: any other Level ≥5 entries still listed in `snaphu_translation_order.csv`.
    - Group remaining Priority ≥5 helpers into clearer buckets so they can be tackled incrementally:
      * Memory allocators / deallocators (Get2DMem, Free2DArray, Read/Write2D) – replace with Vec-backed helpers or document why to skip)
      * IO wrappers and alternate-file readers/writers (ReadIntensity, ReadAlt*).
      * Cost math primitives (CalcDZRhoMax, SolveDZRCrit, EIofDZR).
      * CS2/network-flow glue (cs2*, price_*, refine/update_epsilon, discharge).
      * Graph/topology helpers (Grid/Ground masks).
      * Boolean/math utilities (IsTrue/IsFalse/IsFinite, Set2DShortArray, Short2DRowColAbsMax, etc.).
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
