---
argument-hint: [function_name]
description: Translate a SNAPHU C function to Rust
---

You are a Rust engineer translating a single SNAPHU C function $1 from `snaphu_original/snaphu_full.c` into the safe, idiomatic architecture sketched in `PLAN.md`. Produce a self-contained work order that explains how to re-implement that specific function in Rust.

## How to list functions in the translation order

- Use the command `just translation_plan --status EMPTY` to list all functions in the C source which weren't implemented yet.

## Follow these steps before coding
1. **Identify and understand the C function**
   - Locate the definition inside `snaphu_original/snaphu_full.c` and the entry in `snaphu_translation_order.csv` to capture its priority and dependencies.
   - Summarize what the function currently does, who calls it, and whether it is part of the CLI/config, raster I/O, cost building, tiling, or network/flow subsystems spelled out in `PLAN.md`.
2. **Choose the Rust destination**
   - Map the responsibility to the module tree proposed in the plan (e.g. `src/cli.rs`, `src/context.rs`, `src/config/defaults.rs`, `src/data/raster.rs`, `src/costs/*`, `src/unwrapping/{lpn,tiles,flow}.rs`, `src/network/*.rs`, `src/io/*.rs`).
   - If it manipulates shared runtime data, decide whether the logic belongs on `SnaphuContext`, `RunConfig`, `RuntimeState`, or `TileWorkspace`.
3. **Inventory inputs**
   - List every parameter, macro and global the C version touches. Categorize them as explicit arguments, config/state structs, or transient buffers.
   - Note required helper functions or lookup tables.
4. **Define outputs & side effects**
   - Describe the return value (if any) and which pieces of context the Rust version mutates. Include file I/O, graph updates, or histograms.
   - Call out invariants that must be upheld (array lengths, tile boundaries, cost-scaling constraints, etc.).
5. **Plan the Rust API**
   - Propose a Rust signature (function, method, or struct impl) and explain how it manipulates its inputs/outputs without resorting to globals.
   - Mention any supporting enums/structs/traits that should live alongside it per the plan (e.g. cost calculators in `src/costs`, network builders in `src/network`).
   - Translate original C comments to the new Rust version.
6. **Validation hooks**
   - Suggest how to test the translated function (unit test, fixture produced by `examples/complex_writer.rs`, comparison against `snaphu-sys`).

## Response template for each function $1
```
Function: $1
Priority: {{priority_from_csv}}
C Summary: {{1-2 lines describing behaviour}}
Rust Module Target: {{e.g. src/unwrapping/tiles.rs}}
Rust API Sketch:
    {{pub fn ... signature and ownership story}}
Inputs Needed:
- {{Explicit argument A}} — purpose
- {{Global/Context field}} — map to RunConfig/RuntimeState/TileWorkspace
Outputs & Side Effects:
- {{Return value or struct mutations}}
- {{Context or file writes}}
Implementation Notes:
- {{Key translation details, invariants, helper calls}}
Testing Strategy:
- {{How to confirm parity}}
```

Use this prompt before touching any code: fill it out for the function you intend to port, confirm the Rust destination path, enumerate inputs (including former globals), and pin down outputs so the implementation can proceed without hidden state.
