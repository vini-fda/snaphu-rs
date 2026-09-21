# snaphu-rs raster file formats

Every raster that snaphu-rs reads or writes is raw binary in **native byte
order** (little-endian on x86-64 and Apple silicon). Samples are 4-byte `f32`
values stored **row-major**: row 0 first, left to right, then row 1, and so on.
No format carries a coordinate system, a nodata value, or a `.hdr` sidecar —
those belong to whatever wrote the file.

The formats below are selected per file with config keys (`INFILEFORMAT`,
`OUTFILEFORMAT`, …), either in a config file passed with `-f`, or inline with
`-C "KEY VALUE"`.

## Where the dimensions come from

For the SNAPHU formats the dimensions are *not* in the file:

- **columns (`n`)**: the `linelength` positional argument, e.g.
  `snaphu wrapped.f32 1024`. SNAPHU calls it the line length, i.e. the number
  of samples per line.
- **rows (`m`)**: derived from the file size —
  `m = filesize / (n * bytes_per_sample_pair)`. A file whose size is not an
  exact multiple of one line is rejected with `extra data in file … (bad
  linelength?)`.

Every auxiliary raster (correlation, amplitude, magnitude, mask, weights) must
have the same `m x n` shape as the input interferogram.

The snaphu-rs [`FLOAT_DATA_PHASE_FORMAT`](#float_data_phase_format-phase) is the
exception: it stores `m` and `n` in a header, so the `linelength` argument is
optional for it.

## Format summary

| Token | Bands | Bytes per pixel | Read | Write | Origin |
| --- | --- | --- | --- | --- | --- |
| `COMPLEX_DATA` | real + imaginary | 8 | yes | no | SNAPHU C |
| `FLOAT_DATA` | one | 4 | yes | yes | SNAPHU C |
| `ALT_LINE_DATA` | two, interleaved by line | 8 | yes | yes | SNAPHU C |
| `ALT_SAMPLE_DATA` | two, interleaved by sample | 8 | yes | yes | SNAPHU C |
| `FLOAT_DATA_PHASE_FORMAT` | one, plus a 64-byte header | 4 | yes | yes | snaphu-rs only |

Defaults, if you set nothing: `INFILEFORMAT COMPLEX_DATA`,
`OUTFILEFORMAT ALT_LINE_DATA`, `UNWRAPPEDINFILEFORMAT ALT_LINE_DATA`,
`CORRFILEFORMAT ALT_SAMPLE_DATA`, `AMPFILEFORMAT ALT_SAMPLE_DATA`,
`MAGFILEFORMAT FLOAT_DATA`.

### `COMPLEX_DATA`

Interleaved complex samples: `re0, im0, re1, im1, …`. Magnitude and wrapped
phase are derived as `hypot(re, im)` and `atan2(im, re)`. Input only.

### `FLOAT_DATA`

A plain `m x n` array of `f32`, one band, no header:

```text
row 0: p[0][0] p[0][1] … p[0][n-1]
row 1: p[1][0] …
```

For the interferogram input this band is the wrapped phase, and magnitude
defaults to 1.0 everywhere (no magnitude information is available unless you
also pass `MAGFILE`). For output this band is the unwrapped phase; magnitude is
not written.

### `ALT_LINE_DATA`

Two bands interleaved a whole line at a time — magnitude line, phase line,
magnitude line, … — so the file is `2 * m * n * 4` bytes. This is the SNAPHU
default output format.

### `ALT_SAMPLE_DATA`

Two bands interleaved sample by sample: `a0, b0, a1, b1, …`. For correlation
and amplitude files the second value of each pair is the one SNAPHU uses.

## `FLOAT_DATA_PHASE_FORMAT` (`.phase`)

A snaphu-rs extension: the same single band of `f32` as `FLOAT_DATA`, but
behind a small self-describing header, so a `.phase` file knows its own shape.

**The original SNAPHU C program cannot read or write it.** Use `FLOAT_DATA` for
anything that has to interoperate with the C implementation.

### Layout

```text
byte offset  size    contents
0            4       u32 nrows (m), native endian
4            4       u32 ncols (n), native endian
8            56      reserved, must be zero
64           4*m*n   f32 samples, native endian, row-major
```

Total file size is exactly `64 + m * n * 4` bytes.

- The header is padded to **64 bytes** so the sample block begins on a 64-byte
  boundary: `f32`-aligned, and cache-line/SIMD friendly for readers that
  memory-map the file. The 56 reserved bytes are written as zeros; readers
  ignore them today, so they stay available for future fields.
- `m` and `n` are each `u32`, so their product can exceed 2^32. snaphu-rs does
  every size computation in `u64` with checked arithmetic and reports an error
  rather than wrapping. A shape that is representable but does not match the
  actual file size is also an error.
- `m = 0` or `n = 0` is rejected; the unwrapper additionally needs at least a
  2x2 raster.
- Byte order is native, like every other SNAPHU raster, so `.phase` files are
  not portable between hosts of opposite endianness.

### Using it

`FLOAT_DATA_PHASE_FORMAT` is accepted anywhere a single-band float raster is:
`INFILEFORMAT`, `UNWRAPPEDINFILEFORMAT` (also used for `ESTIMATEFILE`),
`OUTFILEFORMAT`, `CORRFILEFORMAT`, `AMPFILEFORMAT` and `MAGFILEFORMAT`.

```bash
# .phase in, .phase out; the width argument is optional here
snaphu -s \
  -C "INFILEFORMAT FLOAT_DATA_PHASE_FORMAT" \
  -C "OUTFILEFORMAT FLOAT_DATA_PHASE_FORMAT" \
  -o unwrapped.phase wrapped.phase

# mixing containers is fine: .phase in, plain floats out
snaphu -s \
  -C "INFILEFORMAT FLOAT_DATA_PHASE_FORMAT" \
  -C "OUTFILEFORMAT FLOAT_DATA" \
  -o unwrapped.f32 wrapped.phase
```

If you do pass a width, it must agree with the header; a mismatch is reported
instead of silently reshaping the data. As with `FLOAT_DATA`, a `.phase` input
carries no magnitude (it defaults to 1.0), and a `.phase` output holds the
unwrapped phase only.

### Reading and writing it from Rust

```rust
use snaphu_rs::data::raster::Raster;
use snaphu_rs::io::phase_format::{read_phase_header, read_phase_raster, write_phase_file};

let dims = read_phase_header(path)?;          // just the shape
let raster: Raster<f32> = read_phase_raster(path)?;  // shape + samples
write_phase_file(&raster, out_path)?;
```

`snaphu_rs::io::phase_format::read_phase_file` reads a tile window out of a
larger `.phase` file, mirroring `read_2d_array` for the headerless formats.

### Reading and writing it from Python

```python
import numpy as np

def write_phase(path, array):            # array: 2-D float32, row-major
    array = np.ascontiguousarray(array, dtype=np.float32)
    header = np.zeros(64, dtype=np.uint8)
    header[:8] = np.array(array.shape, dtype=np.uint32).view(np.uint8)  # (rows, cols)
    with open(path, "wb") as fp:
        fp.write(header.tobytes())
        fp.write(array.tobytes())

def read_phase(path):
    with open(path, "rb") as fp:
        header = fp.read(64)
        rows, cols = np.frombuffer(header, dtype=np.uint32, count=2)
        return np.fromfile(fp, dtype=np.float32, count=int(rows) * int(cols)).reshape(rows, cols)
```

`np.uint32`/`np.float32` use the machine's native byte order here, which is what
the format requires.

### Example file

`testcases/phase_format/original_phase_example.phase` is an 8x8 `.phase` file
(320 bytes) holding *unwrapped* phase — the ground truth for a small synthetic
test. `tests/phase_format.rs` wraps it, unwraps it again through the CLI, and
checks that the result differs from the wrapped input only by whole cycles and
that it matches the `FLOAT_DATA` path exactly.
