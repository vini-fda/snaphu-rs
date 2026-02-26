# SNAPHU: C to Rust Translation Plan

The end-goal is to hand-write a safe, idiomatic Rust implementation that mirrors SNAPHU's C behaviour while isolating state and depending on the published `cost_scaling_rs` crate instead of the vendored CS2 sources. This document lists the constraints, outlines the target Rust architecture, and defines an incremental translation workflow that follows the dependency order captured in `snaphu_translation_order.csv`. The bundled `examples/complex_writer/` generator is incorporated so we can visualize wrapped interferograms, and capture authoritative fixtures that feed the translation effort. Look at `cargo run --example complex_writer -- --help` for further detail.

## Guiding Principles
- **Single source of truth for state**: Replace global variables with explicit structs (e.g. `SnaphuContext`, `UnwrapConfig`, `TileState`). Pass immutable borrows where possible; constrain mutability to focused subsystems.
- **Prefer inlineable functions or consts over macros**: Translate C macros into enum/struct methods, helper functions, and consts. Use Rust traits when behaviour depends on type families (e.g. cost calculators or CLI option parsing).
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
- Add to `Cargo.toml` using the actual package name/source (current workspace setup): `cost_scaling_rs = { package = "cost-scaling-rs", path = "../cost_scaling_rs" }`.
- Wrap the crate behind `network::MinCostFlowSolver` trait so we can stub it when validating early modules.
- Translate SNAPHU's `InitCs2`, `SolveMCF`, etc. by mapping their inputs to the solver’s struct-based API.
- Keep CS2-specific config (`MAXARCS`, scaling tolerances) expressed as Rust consts managed by the network module.

## Translation Workflow (driven by `snaphu_translation_order.csv`)

The CSV file `snaphu_translation_order.csv` contains a list of all the functions in the original SNAPHU C source code, topologically sorted by the call graph ordering. The "Priority" column in the CSV goes from 0 to 9, where the top-level functions in the call graph are 0, while the leaf nodes have priority 9.

0. **Prepare the `complex_writer` example** *(DONE)*
   - `complex_writer` now lives under `examples/complex_writer.rs` with a clap-powered CLI (`--width/--height`, `--wraps`, `--snaphu-bin`, `--snaphu-arg`, `--skip-snaphu`, `--rerun`, `--write-png`). Running `cargo run --example complex_writer -- …` emits `wrapped_phase.bin`, `snaphu.out`, and (optionally) a PNG preview inside `target/examples/complex_writer/`.
   - The tool accepts a configurable SNAPHU binary path instead of relying on a hard-coded absolute path, and forwards extra arguments so we can exercise different C features while gathering fixtures.
   - Rerun logging, PNG dumps, and README instructions are in place, so the example doubles as a visualization harness and a reproducible dataset generator for parity tests.
1. **Project scaffolding** *(DONE)*
   - The crate compiles quickly with the native Rust CLI path, giving us a clean slate for the idiomatic rewrite while preserving c2rust outputs as reference material.
   - Placeholder modules (`cli`, `config`, `context`, `data`, `costs`, `network`, `unwrapping`, `io`) exist with minimal structs so the new architecture can be filled in incrementally.
2. **Leaf utilities (Priority ≥5)** *(DONE for translatable entries)*
   - ✅ Arithmetic kernels now live under `data::ops` with unit coverage for `Add2DFloatArrays`, `BoxCarAvg`, `AvgSigSq`, and `LClip`.
   - ✅ Lookup-table builders `BuildDZRCritLookupTable` and `BuildDZRhoMaxLookupTable` are implemented in `costs::lookup` with typed parameter structs and smoke tests.
   - ✅ Bucket helpers mirroring `BucketInsert/Remove` exist in `network::bucket` with invariants enforced by Rust errors/tests.
   - ✅ Mirror padding (`MirrorPad`) and interpolation helpers (`LinInterp1D/2D`) are available under `data::ops` with regression tests.
   - ✅ Phase wrapping/sign helpers (`FlattenWrappedPhase`, `WrapPhase`, `ModDiff`, `FlipPhaseArraySign`) now live in `data::ops`, keeping wrapped values inside `[0, 2π)` and matching the short-cycle behaviour from the C code.
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
   - ✅ Complex-input reader `ReadComplexFile` now lives in `io::reader`, converting interleaved real/imag samples into magnitude plus wrapped phase rasters with C-compatible edge handling.
   - ✅ Weight reader `ReadWeightsFile` now lives in `io::reader`, supporting uniform default weights and clipping negative on-disk weights to zero like the C implementation.
   - ✅ Packed arc readers `Read2DRowColFile`/`Read2DRowColFileRows` are available in `io::reader` as typed row/column extraction helpers, preserving block offsets and tile slicing rules from the C implementation.
   - ✅ Tree frontier helper `AddNewNode` now lives in `network::mod`, with explicit bucket-window semantics (`minind/maxind/curr`) and regression tests for reinsert, underflow, overflow, and predecessor-forced updates.
   - ✅ Flow/residue helpers (`CalcFlow`, `CycleResidue`, `NodeResidue`, `IntegratePhase`, `ExtractFlow`, `FlipFlowArraySign`) are now implemented in `data::ops` with row/column-layout parity tests.
   - ✅ Region-growing cost smoother `ThickenCosts` now exists in `costs::types`, preserving row/column arc convolution and `LARGESHORT` clipping semantics.
   - ✅ EI/intensity helpers (`RemoveMean`, `SolveEIModelParams`) are now translated with typed Rust APIs (`data::ops` + `costs::lookup`) and validation coverage.
   - ✅ IO wrappers (`ReadMagnitude`, `ReadByteMask`, `ReadUnwrappedEstimateFile`, `ReadIntensity`, `ReadCorrelation`, `Write2DArray`, `Write2DRowColArray`, `WriteAltLineFile`, `WriteAltSampFile`) now exist as safe typed readers/writers in `io::{reader,writer}`.
   - ✅ Additional graph helpers (`ClosestNode`, `RegionsNeighborNode`, `ScanRegion`, `CheckLeaf`, `CheckBoundary`) are now available in `network::mod` with explicit traversal/consistency tests.
   - ✅ Network topology selectors (`SetGridNetworkFunctionPointers`, `SetNonGridNetworkFunctionPointers`) are represented in `network::mod` with explicit typed mode selection.
   - ✅ Tree/masking network helpers (`CheckMagMasking`, `MaskNodes`, `MaxNonMaskFlow`, `InitNodeNums`, `InitNodes`, `InitBuckets`, `MinOutCostNode`, `FindApex`, `ClipFlow`, `ClearBuckets`) are translated in `network::mod` with row/col-layout and bucket-state parity tests.
   - ✅ `TraceSecondaryArc` now has a typed translation in `unwrapping::tiles`, split into cost-profile tracing and secondary-graph registration helpers, with convergence/zero-cost/reuse path tests.
   - ✅ Config parsing helpers `StringToDouble`/`StringToLong`/`SetBooleanSignedChar` now live in `config::mod`, preserving SNAPHU's full-string parse checks, infinity/overflow guards, legacy empty-string edge behavior, and signed-char boolean assignment semantics.
   - ✅ Runtime/configuration helpers (`CatchSignals`, `StartTimers`, `DisplayElapsedTime`, `LogStringParam`, `LogBoolParam`, `LogFileFormat`, `GetNLines`, `MakeTileDir`, `SetTileInitOutfile`, `SetUpDoTileMask`) are now translated across `cli`, `io::{reader,writer}`, and `unwrapping::tiles` with parity-oriented tests.
   - ✅ Stream/config and core IO orchestrators (`SetStreamPointers`, `SetVerboseOut`, `ChildResetStreamPointers`, `SetDumpAll`, `WriteOutputFile`, `ReadInputFile`, `SetTileReadParams`, `ParseConfigLine`, `ReadConfigFile`, `WriteConfigLogFile`) are translated into typed Rust APIs in `cli`, `io::{reader,writer}`, `config`, and `unwrapping::tiles`, with unit tests for logging, dump defaults, format dispatch, tile-window trimming, and config parsing/log output.
   - ✅ Additional network/tile cost helpers (`DumpIncrCostFiles`, `EvaluateTotalCost`, `CalcInitMaxFlow`, `CheckArcReducedCost`, `ReCalcCost`, `SetupIncrFlowCosts`, `MergeRegions`, `RenumberRegion`, `FindNumPathsOut`, `SetupTile`) are now translated with typed Rust APIs in `io::writer`, `network::mod`, and `unwrapping::tiles`, including clipping/reduced-cost checks, connected-region relabeling, tile temp-name setup, and row/col incremental-cost dump coverage.
   - ✅ Tree/network orchestration helpers (`InitNetwork`, `SetupTreeSolveNetwork`, `SelectSources`, `SelectConnNodeSource`, `MCFInitFlows`, `MSTInitFlows`, `InitTree`, `PruneTree`, `DischargeBoundary`, `CleanUpBoundaryNodes`) are now translated in `network::mod` with safe typed data layouts, boundary discharge cleanup support, and selection/setup routines that reset/prepare tree solve state explicitly.
   - ✅ Tile-region tracing and seam-integration helpers (`ReadNextRegion`, `ReadEdgesAboveAndBelow`, `TraceRegions`, `RegionTraceCheckNeighbors`, `SetUpperEdge`, `SetLowerEdge`, `SetLeftEdge`, `SetRightEdge`, `IntegrateSecondaryFlows`, `ParseSecondaryFlows`) are now translated in `unwrapping::tiles` as safe typed workflows over explicit tile snapshots, edge-flow builders, region traversal state, and secondary-flow parsing/integration APIs.
   - ✅ Priority-0/1 orchestration and Lp/non-grid cost primitives (`CalcCostLP`, `CalcCostLPBiDir`, `CalcCostNonGrid`, `EvalCostLP`, `EvalCostLPBiDir`, `EvalCostNonGrid`, `SetDefaults`, `ProcessArgs`, `CheckParams`, `Unwrap`) are now translated across `unwrapping::{lpn,flow}`, `cli`, and `config` with typed defaults/argument validation and plan-level unwrap scheduling.
   - ✅ Remaining tile/cost/network orchestration entries (`AssembleTiles`, `AssembleTileConnComps`, `BuildCostArrays`, `BuildStatCostsTopo`, `BuildStatCostsDefo`, `BuildStatCostsSmooth`, `GetIntensityAndCorrelation`, `GrowRegions`, `GrowConnCompsMask`, `UnwrapTile`, `SolveCS2`, `SolveMST`, `DischargeTree`, `InitBoundary`, `NonDegenUpdateChildren`, `TreeSolve`) are now translated with safe typed APIs across `unwrapping::{tiles,flow}`, `costs`, and `network`.
   - All CSV entries are now either `DONE` or explicitly `NOT_PLANNED` (17 low-level allocator/CS2 internals intentionally deferred).
3. **Cost builders (Priority 4)** *(DONE for current translation scope)*
   - ✅ `BuildStatCostsTopo`, `BuildStatCostsDefo`, `BuildStatCostsSmooth`, `GetIntensityAndCorrelation`, and `BuildCostArrays` are translated in `costs`.
   - ⏳ Fixture-level numeric parity calibration against the C binary is still pending.
4. **Tile assembly & graph prep (Priority 3)** *(PARTIALLY DONE)*
   - ✅ `AssembleTiles`, `AssembleTileConnComps`, `BuildCostArrays`, `GrowRegions`, and `GrowConnCompsMask` are translated with typed APIs.
   - ⏳ `BuildCostArraysNonGrid` and a fuller `TileGraph` conversion surface for solver backends are still pending.
5. **Network + flow (Priority 2)** *(DONE for current translation scope)*
   - ✅ Typed wrappers/orchestration for `SolveCS2`, `SolveMST`, `DischargeTree`, `InitBoundary`, `NonDegenUpdateChildren`, and `TreeSolve` are present.
   - ✅ `solve_cs2` is wired to `cost_scaling_rs::McmfCs2` (node supplies, arc construction, solver execution, and row/col flow remapping).
6. **Entry points (Priority 0–1)** *(IN PROGRESS)*
   - ✅ High-level translated orchestration exists (`CalcCostLP*`, `CalcCostNonGrid`, CLI/config parsing, `Unwrap`, `UnwrapTile`).
   - ✅ `run_cli` now has a native Rust execution path (`--no-default-features`) that parses arguments, reads input rasters, runs single-tile unwrap, and writes output.
   - ✅ CLI binary target (`src/bin/snaphu.rs`) added with `[[bin]]` in `Cargo.toml`; `cargo run --no-default-features -- -h` prints the full help text matching the C `snaphu -h` output.
   - ✅ `-f` and `-C` flags now read config files and inline config strings via `apply_config_entries()`. File format fields (`INFILEFORMAT`, `OUTFILEFORMAT`, `CORRFILEFORMAT`, etc.) are tracked in `RunConfig` and used by the native execution path instead of hardcoded values.
   - ✅ Added a dedicated performance-first multi-tile execution roadmap in `MULTI_TILE.md` (threaded scheduling, deterministic assembly, and future memory/disk tile-store strategy).
   - ✅ Multi-tile Phase 1 implemented in `unwrapping::multitile`: parallel tile unwrap with atomic work queue, `OnceLock` result slots, bulk offset propagation via `set_right_edge`/`set_lower_edge`, and assembly via `integrate_secondary_flows`. CLI now routes `--tile` configs to the native Rust multi-tile path (only `-S` and `--assemble` still rejected).
   - ✅ Native multi-tile secondary-network optimization is now active: tiles trace deterministic cross-boundary secondary arcs into a shared graph, normalize arc costs, solve global secondary flows, map solved arc flows back per tile, and apply parsed secondary-flow corrections during assembly (replacing the previous stubbed empty-graph/empty-flow path).
   - ✅ Multi-tile follow-up optimizations landed: reusable worker-local extraction buffers (`mag`/`wrapped`/`power`/`correlation`), stricter seam-length validation (no silent truncation), safer tile-window bounds checks, and verbose per-stage timing for profiling.
   - ✅ Native multi-tile path now supports `DOTILEMASKFILE`: config parsing/wiring is in place, tile masks are loaded via `set_up_do_tile_mask`, and masked tiles bypass unwrap while still producing deterministic assembled output.
   - ✅ Added a C/Rust parity-debug pass for 600x600 Kumamoto runs: extra DEFO/SMOOTH config knobs are now parsed, cost construction and CS2 graph mapping were aligned with C conventions, and both binaries can emit stage-level debug counters for side-by-side diagnostics.
   - ⏳ Full CLI parity is still pending (native path currently scopes to single/multi-tile non-quantify workflows).

While translating each CSV-priority batch, update the spreadsheet (or a markdown checklist) with statuses so we know which functions remain.

## Handling Macros & Constants
- Convert flag-style macros (`#define USE_NEW_GRAPH 1`) into `const bool` or enums inside the config module.
- Replace inline macro math (e.g. `#define SQR(x) ((x)*(x))`) with `fn sqr(x: f32) -> f32` or `num::Float::powi` where clarity matters. These functions will be inlined
- Multi-line macros that encapsulate logic should become private helper functions with well-typed parameters, making ownership explicit.

## Testing & Validation Strategy
- **Golden fixtures**: Capture small interferogram tiles + expected unwrap outputs from the C binary. Re-run after each major module to prevent drift.
- **Property tests**: For cost calculators and lookup tables, assert monotonicity, ranges, and invariants mirrored from SNAPHU docs.
- **Integration harness**: Keep the original C pipeline (`snaphu_original/snaphu`) runnable as a reference until full parity is achieved.
- **Example-driven datasets**: Use `cargo run --example complex_writer` to regenerate wrapped interferograms on demand, collect the resulting `.bin` / `.out` files, and compare SNAPHU vs SNAPHU-rs outputs to refresh fixtures whenever upstream logic evolves.

## Risk & Mitigation
- **Global-state coupling**: Some functions rely on implicit writes; when translating, add temporary logging to the C version to trace which globals each function touches.
- **Floating-point drift**: Document acceptable tolerances per module; prefer `f64` for accumulators even if C used double.
- **CS2 parity**: `cost_scaling_rs` may expose different tuning knobs; add adapter layer to emulate SNAPHU defaults and keep integration tests focused on network outputs instead of internal solver steps.

## Verification Snapshot (2026-02-22)
- `just translation_plan --status EMPTY` => no rows.
- `just translation_plan --status DONE` => 173 rows.
- `just translation_plan --status NOT_PLANNED` => 17 rows (deferred low-level allocator/CS2 internals).
- `cargo test --no-default-features` passes (native CLI smoke tests included).
- `Cargo.toml` includes `cost_scaling_rs` (package `cost-scaling-rs`) and `solve_cs2` is backed by the solver integration.

## Definition of Done
1. `cargo test --no-default-features` passes with only the idiomatic Rust implementation.
2. CLI arguments, config file semantics, and output formats match the original binary (validated via regression suite).
3. All functions listed in `snaphu_translation_order.csv` are marked `DONE` or `NOT_PLANNED` with explicit rationale.
