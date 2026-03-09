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
  lib.rs                # Public API + CLI entry
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
tests/                  # integration tests
    common/             # common testing utils
    kumamoto.rs         # full test with kumamoto earthquake data
    rerun_*.rs          # manual tests using the Rerun GUI for visualization
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
- Translate SNAPHU's `InitCs2`, `SolveMCF`, etc. by mapping their inputs to the solver’s struct-based API.

## Handling Macros & Constants
- Convert flag-style macros (`#define USE_NEW_GRAPH 1`) into `const bool` or enums inside the config module.
- Replace inline macro math (e.g. `#define SQR(x) ((x)*(x))`) with `fn sqr(x: f32) -> f32` or `num::Float::powi` where clarity matters. These functions will be inlined
- Multi-line macros that encapsulate logic should become private helper functions with well-typed parameters, making ownership explicit.

## Testing & Validation Strategy
- **Golden fixtures**: Capture small interferogram tiles + expected unwrap outputs from the C binary. Re-run after each major module to prevent drift.
- **Property tests**: For cost calculators and lookup tables, assert monotonicity, ranges, and invariants mirrored from SNAPHU docs.
- **Example-driven datasets**: Use the kumamoto dataset with the corresponding full end to end comparison test in `tests/kumamoto.rs` as a validation.

## Risk & Mitigation
- **Global-state coupling**: Some functions rely on implicit writes; when translating, add temporary logging to the C version to trace which globals each function touches.
- **Floating-point drift**: Document acceptable tolerances per module; prefer `f64` for accumulators even if C used double.
- **CS2 parity**: `cost_scaling_rs` may expose different tuning knobs; add adapter layer to emulate SNAPHU defaults and keep integration tests focused on network outputs instead of internal solver steps.

## Definition of Done
1. `cargo test --no-default-features` passes with only the idiomatic Rust implementation.
2. CLI arguments, config file semantics, and output formats match the original binary (validated via regression suite).
3. All functions listed in `snaphu_translation_order.csv` are marked `DONE` or `NOT_PLANNED` with explicit rationale.
