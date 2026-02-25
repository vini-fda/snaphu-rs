# snaphu-rs

[![crates.io](https://img.shields.io/crates/v/snaphu-rs.svg)](https://crates.io/crates/snaphu-rs) [![docs.rs](https://docs.rs/snaphu-rs/badge.svg)](https://docs.rs/snaphu-rs)

This is a Rust-based implementation of the [SNAPHU](https://web.stanford.edu/group/radar/softwareandlinks/sw/snaphu/) library,
a C implementation of a statistical-cost, network-flow algorithm for two-dimensional phase unwrapping.

This library is still heavily **work-in-progress**. The plan is to have a library that closely matches the original C implementation, but
using safe and idiomatic Rust idioms, with proper documentation, and an easy-to-use API.

The emphasis is first on **correctness**, then performance. So tests comparing both implementations should be implemented soon.

## Generating synthetic interferograms

The repository ships with a helper example that creates a synthetic interferogram, writes it to disk, and (optionally) runs the original `snaphu` binary so we can capture golden outputs for the Rust port.

```
cargo run --example complex_writer -- \
    --out-dir target/examples/complex_writer \
    --snaphu-bin /path/to/snaphu \
    --rerun --write-png
```

- `--snaphu-bin` defaults to `snaphu` on your `$PATH`; pass `--skip-snaphu` to generate data without invoking the C binary.
- Re-run with different sizes (`--width/--height`) or `--snaphu-arg` flags to explore other SNAPHU modes.
- The command writes `wrapped_phase.bin`, `snaphu.out`, and (optionally) `wrapped_phase.png` under the chosen output directory. These files double as fixtures when comparing SNAPHU vs `snaphu-rs`.
- Use `--rerun` to stream the wrapped/unwrapped fields into a [Rerun](https://rerun.io/) viewer for quick inspection while iterating on the Rust translation.

These datasets will form the initial regression corpus so we can prove parity as the idiomatic Rust implementation replaces the temporary c2rust bindings.

## License

This project includes code derived from SNAPHU and the cs2 minimum-cost
flow solver.

The cs2 component is licensed for **non-commercial / evaluation use only**.
As a result, this crate may not be used for commercial purposes.

See the original SNAPHU and cs2 documentation for full terms.

## References

This library is based on the phase unwrapping algorithms developed by Curtis W. Chen and Howard A. Zebker, as described in the following publications:
1.	C. W. Chen and H. A. Zebker,_Network approaches to two-dimensional phase unwrapping: intractability and two new algorithms_,Journal of the Optical Society of America A, Vol. 17, pp. 401–414 (2000).
2.	C. W. Chen and H. A. Zebker,_Two-dimensional phase unwrapping with use of statistical models for cost functions in nonlinear optimization_,Journal of the Optical Society of America A, Vol. 18, pp. 338–351 (2001).
3.	C. W. Chen and H. A. Zebker,_Phase unwrapping for large SAR interferograms: Statistical segmentation and generalized network models_,IEEE Transactions on Geoscience and Remote Sensing, Vol. 40, pp. 1709–1719 (2002).
