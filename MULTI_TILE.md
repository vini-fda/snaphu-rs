# Multi-Tile Unwrapping Plan (Performance-First)

## Context

The native Rust CLI currently hard-stops when tile mode is requested in [`src/lib.rs`](/Users/vinifreitas/Programming/snaphu-rs/src/lib.rs:245).  
Kumamoto uses `--tile 10 10 ...`, so this blocks a key real-world workload and prevents direct runtime/quality comparison against C SNAPHU.

The C reference uses `fork()` and file-backed tile workflows.  
For Rust, we should use a memory-first multithreaded design for speed, with optional disk spill for large scenes.

## Goals

1. Enable native Rust multi-tile execution in CLI.
2. Maximize throughput for large tile grids (e.g., 10x10+).
3. Keep memory bounded via optional spill-to-disk mode.
4. Preserve deterministic output mode for regression tests.

## Non-Goals (Phase 1)

1. Full C-equivalent secondary network optimization across tiles.
2. `-S` one-tile reoptimization.
3. `--assemble` compatibility with legacy C tile files.

## Key Design Decisions

1. **Concurrency model**
   - Use `std::thread::scope` with `worker_count = min(params.nthreads, ntiles)`.
   - Use an `AtomicUsize` index queue (`fetch_add`) instead of `Mutex<Iterator>`.
   - Use a pre-sized output slot array (`Vec<OnceLock<TileResult>>`) to avoid result-vector lock contention.

2. **Data movement**
   - Do **not** allocate a fresh `Vec<Vec<f32>>` per tile job.
   - Use thread-local reusable buffers per worker for tile `mag`/`phase`/optional `corr`/`power`.
   - Keep full-scene data loaded once; workers copy only windowed tile regions into reusable buffers.

3. **Storage policy**
   - Start with in-memory tile outputs (`TileResult` structs).
   - Add optional spill backend (`--tile-store auto|memory|disk`, later phase):
     - `memory`: fastest.
     - `disk`: low-RAM fallback.
     - `auto`: memory until budget, then spill cold tiles.

4. **Assembly**
   - Reuse existing typed tile APIs in [`src/unwrapping/tiles.rs`](/Users/vinifreitas/Programming/snaphu-rs/src/unwrapping/tiles.rs):
     - `setup_tile`
     - `set_right_edge`
     - `set_lower_edge`
     - `integrate_secondary_flows` / `assemble_tiles`
   - Phase 1 assembly uses bulk offsets only (no secondary MCF arcs yet).

5. **Determinism and observability**
   - Keep deterministic final ordering by tile index (`tilerow * ntilecol + tilecol`), independent of completion order.
   - Add optional verbose per-stage timing and per-tile timing.

## Implementation Plan

## Phase 0: Refactor `run_cli` into explicit stages

1. Split existing single-tile path in [`src/lib.rs`](/Users/vinifreitas/Programming/snaphu-rs/src/lib.rs) into helpers:
   - `read_scene_inputs(...)`
   - `run_single_tile(...)`
   - `run_multi_tile(...)`
2. Replace current guard at line ~245 with:
   - reject only unsupported modes (`onetilereopt`, `assemble_only`, `eval`) for now.
   - route based on `multi_tile = params.ntilerow != 1 || params.ntilecol != 1`.

## Phase 1: Multi-tile baseline (high performance, no secondary MCF)

1. **Precompute tile grid**
   - Build `TileSetupParams` and call `setup_tile(...)` once for every tile.
   - Store `TilePlanEntry { tilerow, tilecol, region, tilenum }` in raster order.

2. **Parallel tile unwrap**
   - Spawn scoped workers with atomic work distribution.
   - Worker loop per tile:
     1. Fill thread-local buffers from full-scene rasters using tile region.
     2. Call `unwrap_tile(...)`.
     3. Convert row/col flows to flat flows, then call `integrate_phase(...)`.
     4. Store `TileResult` in its pre-assigned slot.
   - `TileResult` should minimally include:
     - tile geometry (`TileRegion`)
     - `mag`, `unw_phase`, `regions`
     - placeholder `arc_indices`, `secondary_flows` (empty in Phase 1)

3. **Bulk offset propagation**
   - Allocate `bulk_offsets[ntilerow][ntilecol]` initialized to `0`.
   - Traverse tiles in raster order:
     - call `set_right_edge` for right neighbors
     - call `set_lower_edge` for below neighbors
   - Use tile border vectors from `TileResult.unw_phase`.

4. **Global assembly**
   - Build `Vec<TileIntegrationInput>` in deterministic tile order.
   - Call `integrate_secondary_flows(...)` (or `assemble_tiles(...)`) with:
     - `TileReadSettings` from params
     - computed `bulk_offsets`
     - empty graph/secondary flows for now
   - Convert assembled output to `Raster<f32>` and write with `write_output_file(...)`.

5. **Error/cancel behavior**
   - Add shared `AtomicBool failed`.
   - On first worker failure, set `failed = true`; remaining workers stop pulling new jobs.
   - Return first captured error with tile coordinates.

## Phase 2: Accuracy parity path (secondary MCF over tile graph)

1. Populate non-empty `arc_indices` and `secondary_flows` using traced inter-tile graph.
2. Reuse/extend graph types already present in [`src/unwrapping/tiles.rs`](/Users/vinifreitas/Programming/snaphu-rs/src/unwrapping/tiles.rs).
3. Gate with feature flag or config flag until validated against C.

## Phase 3: Memory/disk hybrid tile store

1. Introduce `TileStore` trait:
   - `put(tile_id, TileResult)`
   - `get(tile_id) -> TileResultRef`
2. Implement:
   - `MemoryTileStore`
   - `DiskTileStore` (binary tile blobs)
   - `HybridTileStore` (LRU spill by memory budget)
3. Add CLI/config knobs:
   - `--tile-store`
   - `--tile-memory-budget-mb`
   - `--tiledir` reuse for disk/hybrid modes

## Concrete File Changes

1. [`src/lib.rs`](/Users/vinifreitas/Programming/snaphu-rs/src/lib.rs)
   - Remove single-tile-only guard.
   - Add `run_multi_tile` orchestration and stage helpers.
   - Add worker scheduler and deterministic collection logic.

2. [`src/unwrapping/tiles.rs`](/Users/vinifreitas/Programming/snaphu-rs/src/unwrapping/tiles.rs)
   - Reuse existing APIs; add small glue helpers only if needed.
   - Avoid changing math primitives unless parity bugs require it.

3. Optional new module: `src/unwrapping/multitile.rs`
   - Hold `TilePlanEntry`, `TileResult`, scheduler, and assembly wiring.
   - Keep `lib.rs` lean.

## Performance Notes (Important)

1. Prefer `--release` benchmarks only.
2. Avoid per-tile heap churn:
   - reuse buffers
   - pre-size vectors
3. Avoid lock hotspots:
   - atomic queue index
   - one-write-per-slot result container
4. Track timings:
   - read/input prep
   - tile unwrap total + per-tile p50/p95/max
   - bulk-offset stage
   - assembly stage
   - output write

## Verification

1. `cargo build --no-default-features --bin snaphu`
2. `cargo test --no-default-features`
3. Kumamoto run:

```bash
cd /Users/vinifreitas/Programming/snaphu-rs/snaphu_input
cargo run --release --no-default-features --bin snaphu -- \
  -f snaphu.conf Phase_ifg_IW2_VV_08Apr2016_20Apr2016.snaphu.img 24639
```

4. Compare to C output:
   - pixel-wise phase delta stats (`mean`, `p95`, `max`)
   - connected-component consistency (when available)
   - runtime breakdown (native Rust vs C)

## Acceptance Criteria

1. Native Rust CLI runs Kumamoto in tile mode without unsupported-mode error.
2. Multi-tile output is produced end-to-end in Rust path.
3. No regressions in existing single-tile behavior/tests.
4. Runtime scales with `--nproc` (up to CPU/core and memory limits).
5. Architecture supports adding secondary MCF and hybrid tile storage without redesign.
