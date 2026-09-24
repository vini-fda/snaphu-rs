//! The snaphu-rs native `.phase` raster container.
//!
//! `.phase` is a snaphu-rs extension (it is *not* understood by the original
//! SNAPHU C program). It wraps a single-band `f32` raster in a small,
//! self-describing header so that the row count and the column count travel
//! with the samples instead of being passed on the command line:
//!
//! ```text
//! byte offset  size  contents
//! 0            4     u32 nrows, native endian
//! 4            4     u32 ncols, native endian
//! 8            56    reserved, zero-filled padding
//! 64           4*n   f32 samples, native endian, row-major (n = nrows*ncols)
//! ```
//!
//! The header is padded to [`PHASE_HEADER_LEN`] bytes so the sample block
//! starts at a 64-byte boundary, which keeps it aligned for `f32` (and for
//! typical SIMD/cache-line access) when a reader memory-maps the file.
//!
//! `nrows` and `ncols` are each `u32`, so their product can exceed `u32::MAX`;
//! every size computation here is done in `u64` (or with checked arithmetic)
//! and is rejected if it cannot be represented on the host.
//!
//! Everything is native endian, exactly like the other SNAPHU raster formats,
//! so a `.phase` file is not portable between hosts of different byte order.

use crate::data::raster::Raster;
use crate::io::reader::{TileWindow, read_2d_array_after_header};
use crate::io::writer::open_output_file;
use std::fs::File;
use std::io::{self, Read};
use std::path::{Path, PathBuf};

/// Total size of the zero-padded `.phase` header, in bytes.
pub const PHASE_HEADER_LEN: usize = 64;

/// Number of header bytes that carry meaning today (`u32` rows, `u32` cols).
///
/// The remaining `PHASE_HEADER_LEN - PHASE_HEADER_USED_LEN` bytes are reserved
/// and must be written as zeros.
pub const PHASE_HEADER_USED_LEN: usize = 8;

/// Conventional file extension for the format.
pub const PHASE_FILE_EXTENSION: &str = "phase";

/// Raster shape declared by a `.phase` header.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhaseDims {
    pub nrows: usize,
    pub ncols: usize,
}

impl PhaseDims {
    /// Number of samples in the file, computed in `u64` because `nrows` and
    /// `ncols` are both `u32` and their product can overflow 32 bits.
    fn sample_count(&self) -> io::Result<u64> {
        let nrows = self.nrows as u64;
        let ncols = self.ncols as u64;
        nrows
            .checked_mul(ncols)
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "sample count overflow"))
    }

    /// Total on-disk size of a `.phase` file with this shape.
    fn expected_file_len(&self) -> io::Result<u64> {
        self.sample_count()?
            .checked_mul(std::mem::size_of::<f32>() as u64)
            .and_then(|bytes| bytes.checked_add(PHASE_HEADER_LEN as u64))
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "file size overflow"))
    }
}

fn read_u32_ne(bytes: &[u8]) -> u32 {
    u32::from_ne_bytes(bytes.try_into().expect("4-byte slice"))
}

/// Read and validate the header of a `.phase` file.
///
/// Fails when the file is too short, declares an empty raster, declares
/// dimensions too large for the host, or has a size that disagrees with the
/// declared shape.
pub fn read_phase_header(path: &Path) -> io::Result<PhaseDims> {
    let mut fp = File::open(path)?;
    let filesize = fp.metadata()?.len();
    if filesize < PHASE_HEADER_LEN as u64 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "file {} is {} bytes, too short for a {}-byte .phase header",
                path.display(),
                filesize,
                PHASE_HEADER_LEN
            ),
        ));
    }

    let mut header = [0u8; PHASE_HEADER_LEN];
    fp.read_exact(&mut header)?;
    let nrows = read_u32_ne(&header[0..4]);
    let ncols = read_u32_ne(&header[4..8]);
    if nrows == 0 || ncols == 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "file {} declares an empty raster ({} x {})",
                path.display(),
                nrows,
                ncols
            ),
        ));
    }

    let to_host = |value: u32, what: &str| {
        usize::try_from(value).map_err(|_| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "file {} declares {} = {}, too large for this host",
                    path.display(),
                    what,
                    value
                ),
            )
        })
    };
    let dims = PhaseDims {
        nrows: to_host(nrows, "nrows")?,
        ncols: to_host(ncols, "ncols")?,
    };

    let expected = dims.expected_file_len().map_err(|err| {
        io::Error::new(
            err.kind(),
            format!(
                "file {} declares {} x {}: {}",
                path.display(),
                dims.nrows,
                dims.ncols,
                err
            ),
        )
    })?;
    if filesize != expected {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "file {} is {} bytes but its header declares {} x {} ({} bytes expected)",
                path.display(),
                filesize,
                dims.nrows,
                dims.ncols,
                expected
            ),
        ));
    }
    // The sample block must also be addressable in memory once read.
    usize::try_from(dims.sample_count()?).map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "file {} declares {} x {}, too many samples for this host",
                path.display(),
                dims.nrows,
                dims.ncols
            ),
        )
    })?;

    Ok(dims)
}

/// Read a tile window from a `.phase` file whose shape must be `nlines x line_len`.
///
/// This is the `.phase` counterpart of
/// [`read_2d_array`](crate::io::reader::read_2d_array): the caller states the
/// shape it expects (as the other SNAPHU readers do) and the header is checked
/// against it.
pub fn read_phase_file(
    path: &Path,
    line_len: usize,
    nlines: usize,
    window: TileWindow,
) -> io::Result<Raster<f32>> {
    let dims = read_phase_header(path)?;
    if dims.ncols != line_len || dims.nrows != nlines {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "file {} declares {} x {} but {} x {} was expected",
                path.display(),
                dims.nrows,
                dims.ncols,
                nlines,
                line_len
            ),
        ));
    }
    read_2d_array_after_header::<f32>(path, line_len, nlines, window, PHASE_HEADER_LEN)
}

/// Read a whole `.phase` file, taking its shape from the header.
pub fn read_phase_raster(path: &Path) -> io::Result<Raster<f32>> {
    let dims = read_phase_header(path)?;
    read_phase_file(
        path,
        dims.ncols,
        dims.nrows,
        TileWindow::new(0, 0, dims.nrows, dims.ncols),
    )
}

/// Write a raster as a `.phase` file, returning the path actually written.
pub fn write_phase_file(raster: &Raster<f32>, outfile: &Path) -> io::Result<PathBuf> {
    if !raster.has_valid_shape() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "raster data length does not match width*height",
        ));
    }
    let nrows = u32::try_from(raster.height).map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            format!(
                "row count {} does not fit in the u32 header field",
                raster.height
            ),
        )
    })?;
    let ncols = u32::try_from(raster.width).map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            format!(
                "column count {} does not fit in the u32 header field",
                raster.width
            ),
        )
    })?;
    if nrows == 0 || ncols == 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "cannot write an empty .phase raster",
        ));
    }

    let mut header = [0u8; PHASE_HEADER_LEN];
    header[0..4].copy_from_slice(&nrows.to_ne_bytes());
    header[4..8].copy_from_slice(&ncols.to_ne_bytes());

    let mut opened = open_output_file(outfile)?;
    {
        use std::io::Write;
        opened.file.write_all(&header)?;
        // Buffer the sample block: a raster can be hundreds of megabytes.
        let mut writer = io::BufWriter::new(&mut opened.file);
        for sample in &raster.data {
            writer.write_all(&sample.to_ne_bytes())?;
        }
        writer.flush()?;
    }
    Ok(opened.real_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_file(prefix: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("{prefix}_{}_{}", std::process::id(), nanos))
    }

    fn write_raw(path: &Path, header_rows: u32, header_cols: u32, samples: &[f32]) {
        let mut header = [0u8; PHASE_HEADER_LEN];
        header[0..4].copy_from_slice(&header_rows.to_ne_bytes());
        header[4..8].copy_from_slice(&header_cols.to_ne_bytes());
        let mut fp = File::create(path).unwrap();
        fp.write_all(&header).unwrap();
        for value in samples {
            fp.write_all(&value.to_ne_bytes()).unwrap();
        }
    }

    #[test]
    fn round_trip_preserves_shape_and_samples() {
        let path = temp_file("snaphu_rs_phase_roundtrip");
        let raster = Raster::new(4, 3, (0..12).map(|v| v as f32 * 0.25).collect());
        let written = write_phase_file(&raster, &path).unwrap();
        assert_eq!(written, path);
        assert_eq!(
            std::fs::metadata(&path).unwrap().len(),
            (PHASE_HEADER_LEN + 12 * 4) as u64
        );

        let dims = read_phase_header(&path).unwrap();
        assert_eq!(dims, PhaseDims { nrows: 3, ncols: 4 });
        assert_eq!(read_phase_raster(&path).unwrap(), raster);

        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn header_reserved_bytes_are_zero_filled() {
        let path = temp_file("snaphu_rs_phase_padding");
        write_phase_file(&Raster::new(2, 2, vec![0.0; 4]), &path).unwrap();
        let bytes = std::fs::read(&path).unwrap();
        assert!(
            bytes[PHASE_HEADER_USED_LEN..PHASE_HEADER_LEN]
                .iter()
                .all(|b| *b == 0)
        );
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn read_phase_file_reads_selected_tile_window() {
        let path = temp_file("snaphu_rs_phase_window");
        let samples: Vec<f32> = (0..12).map(|v| v as f32).collect();
        write_raw(&path, 3, 4, &samples);

        let tile = read_phase_file(&path, 4, 3, TileWindow::new(1, 1, 2, 2)).unwrap();
        assert_eq!(tile.width, 2);
        assert_eq!(tile.height, 2);
        assert_eq!(tile.data, vec![5.0, 6.0, 9.0, 10.0]);

        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn read_phase_file_rejects_shape_mismatch() {
        let path = temp_file("snaphu_rs_phase_mismatch");
        write_raw(&path, 3, 4, &(0..12).map(|v| v as f32).collect::<Vec<_>>());

        let err = read_phase_file(&path, 3, 4, TileWindow::new(0, 0, 4, 3)).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
        assert!(err.to_string().contains("was expected"));

        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn read_phase_header_rejects_truncated_file() {
        let path = temp_file("snaphu_rs_phase_short");
        File::create(&path).unwrap().write_all(&[0u8; 16]).unwrap();
        let err = read_phase_header(&path).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
        assert!(err.to_string().contains("too short"));
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn read_phase_header_rejects_size_mismatch() {
        let path = temp_file("snaphu_rs_phase_badsize");
        // Header claims 3x4 but only 11 samples follow.
        write_raw(&path, 3, 4, &(0..11).map(|v| v as f32).collect::<Vec<_>>());
        let err = read_phase_header(&path).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
        assert!(err.to_string().contains("bytes expected"));
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn read_phase_header_rejects_zero_dimension() {
        let path = temp_file("snaphu_rs_phase_empty");
        write_raw(&path, 0, 4, &[]);
        let err = read_phase_header(&path).unwrap_err();
        assert!(err.to_string().contains("empty raster"));
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn read_phase_header_rejects_dimensions_whose_product_overflows() {
        let path = temp_file("snaphu_rs_phase_huge");
        // 2^31 x 2^31 samples: each dimension fits its u32 field, but the
        // sample count (2^62) times 4 bytes does not fit in u64.
        write_raw(&path, 1 << 31, 1 << 31, &[]);
        let err = read_phase_header(&path).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
        assert!(
            err.to_string().contains("file size overflow"),
            "unexpected error: {err}"
        );
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn read_phase_header_rejects_oversized_but_representable_shape() {
        let path = temp_file("snaphu_rs_phase_big");
        // 1 x 2^30: the declared size is representable, just not this file's.
        write_raw(&path, 1, 1 << 30, &[]);
        let err = read_phase_header(&path).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
        assert!(
            err.to_string().contains("bytes expected"),
            "unexpected error: {err}"
        );
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn expected_file_len_rejects_u64_overflow() {
        let dims = PhaseDims {
            nrows: u32::MAX as usize,
            ncols: u32::MAX as usize,
        };
        // The sample count still fits in u64; the byte count does not.
        assert!(dims.sample_count().is_ok());
        assert!(dims.expected_file_len().is_err());
    }
}
