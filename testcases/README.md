# SNAPHU Testcases (Kumamoto)

This directory contains three fixed-size SNAPHU input testcases exported from the same interferogram:

- `kumamoto_64` (`64 x 64`)
- `kumamoto_256` (`256 x 256`)
- `kumamoto_1024` (`1024 x 1024`)

Each testcase folder has SNAPHU inputs in:

- `kumamoto_<size>/`

Files you need:

- `Phase_ifg_IW2_VV_08Apr2016_20Apr2016.snaphu.img`
- `coh_IW2_VV_08Apr2016_20Apr2016.snaphu.img`
- `snaphu.conf`

## Run SNAPHU (per testcase)

Use the testcase directory as your working directory:

```bash
cd outputs/testcases/kumamoto_64/
snaphu -f snaphu.conf Phase_ifg_IW2_VV_08Apr2016_20Apr2016.snaphu.img 64
```

For the other two testcases, only the last argument changes:

- `kumamoto_256`: width argument `256`
- `kumamoto_1024`: width argument `1024`

SNAPHU width argument = number of samples (columns) in the wrapped phase raster.

## Run all three

```bash
for n in 64 256 1024; do
  d="outputs/testcases/kumamoto_${n}/"
  (
    cd "$d" || exit 1
    snaphu -f snaphu.conf Phase_ifg_IW2_VV_08Apr2016_20Apr2016.snaphu.img "$n"
  )
done
```

## Expected output from SNAPHU

After running, each testcase directory should include:

- `UnwPhase_ifg_IW2_VV_08Apr2016_20Apr2016.snaphu.img`
- `UnwPhase_ifg_IW2_VV_08Apr2016_20Apr2016.snaphu.hdr`
- `snaphu.log`

## `.phase` example

`phase_format/original_phase_example.phase` is a small 8x8 raster of unwrapped
phase in the snaphu-rs `.phase` format (`FLOAT_DATA_PHASE_FORMAT`), used by
`tests/phase_format.rs`. The format is documented in
[../docs/file-formats.md](../docs/file-formats.md).
