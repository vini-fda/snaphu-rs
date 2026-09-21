//! Configuration types for running SNAPHU.

use core::ffi::c_long;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

/// On-disk raster encoding label, mirroring SNAPHU config file tokens.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileFormat {
    ComplexData,
    FloatData,
    AltSampleData,
    AltLineData,
    /// snaphu-rs extension (`FLOAT_DATA_PHASE_FORMAT`): single-band floats
    /// behind a self-describing `.phase` header.
    ///
    /// See [`crate::io::phase_format`]. Files in this format are not readable
    /// by the original SNAPHU C program.
    FloatDataPhase,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CostMode {
    NoStatCosts,
    Topo,
    Defo,
    Smooth,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InitMethod {
    Mst,
    Mcf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransmitMode {
    PingPong,
    SingleAntenna,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct InputFiles {
    pub infile: String,
    pub weightfile: String,
    pub corrfile: String,
    pub ampfile: String,
    pub ampfile2: String,
    pub estfile: String,
    pub magfile: String,
    pub costinfile: String,
    pub bytemaskfile: String,
    pub dotilemaskfile: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutputFiles {
    pub initfile: String,
    pub flowfile: String,
    pub eifile: String,
    pub rowcostfile: String,
    pub colcostfile: String,
    pub mstrowcostfile: String,
    pub mstcolcostfile: String,
    pub mstcostsfile: String,
    pub corrdumpfile: String,
    pub rawcorrdumpfile: String,
    pub costoutfile: String,
    pub conncompfile: String,
    pub outfile: String,
    pub logfile: String,
}

impl Default for OutputFiles {
    fn default() -> Self {
        Self {
            initfile: String::new(),
            flowfile: String::new(),
            eifile: String::new(),
            rowcostfile: String::new(),
            colcostfile: String::new(),
            mstrowcostfile: String::new(),
            mstcolcostfile: String::new(),
            mstcostsfile: String::new(),
            corrdumpfile: String::new(),
            rawcorrdumpfile: String::new(),
            costoutfile: String::new(),
            conncompfile: String::new(),
            outfile: "snaphu.out".to_string(),
            logfile: String::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunConfig {
    pub unwrapped: bool,
    pub regrow_conn_comps: bool,
    pub eval: bool,
    pub init_only: bool,
    pub init_method: InitMethod,
    pub cost_mode: CostMode,
    pub amplitude: bool,
    pub verbose: bool,
    pub bperp: f64,
    pub p: f64,
    pub onetilereopt: bool,
    pub rm_tmp_tile: bool,
    pub rm_tile_init: bool,
    pub dump_all: bool,
    pub ntilerow: usize,
    pub ntilecol: usize,
    pub rowovrlp: usize,
    pub colovrlp: usize,
    pub piecefirstrow: usize,
    pub piecefirstcol: usize,
    pub piecenrow: usize,
    pub piecencol: usize,
    pub nthreads: usize,
    pub assemble_only: bool,
    pub tiledir: String,
    pub parent_pid: i64,
    pub earthradius: f64,
    pub altitude: f64,
    pub orbitradius: f64,
    pub baseline_angle: f64,
    pub transmit_mode: TransmitMode,
    pub baseline: f64,
    pub ncorrlooks: f64,
    pub nearrange: f64,
    pub dr: f64,
    pub da: f64,
    pub range_resolution: f64,
    pub lambda: f64,
    pub kds: f64,
    pub specular_exponent: f64,
    pub dzrcrit_factor: f64,
    pub shadow: bool,
    pub dzeimin: f64,
    pub laywidth: usize,
    pub layminei: f64,
    pub sloperatio_factor: f64,
    pub sigsqei: f64,
    pub drho: f64,
    pub threshold: f64,
    pub initdzr: f64,
    pub initdzstep: f64,
    pub dnomincangle: f64,
    pub costscaleambight: f64,
    pub rhosconst1: f64,
    pub rhosconst2: f64,
    pub cstd1: f64,
    pub cstd2: f64,
    pub cstd3: f64,
    pub defaultcorr: f64,
    pub rhominfactor: f64,
    pub dzlaypeak: f64,
    pub azdzfactor: f64,
    pub dzeifactor: f64,
    pub dzeiweight: f64,
    pub dzlayfactor: f64,
    pub layconst: f64,
    pub defothreshfactor: f64,
    pub defomax: f64,
    pub sigsqcorr: f64,
    pub costscale: f64,
    pub defolayconst: f64,
    pub sigsqshortmin: i64,
    pub sigsqlayfactor: f64,
    pub kperpdpsi: usize,
    pub kpardpsi: usize,
    pub krowei: usize,
    pub kcolei: usize,
    pub maxcost: i64,
    pub layfalloffconst: i64,
    pub nshortcycle: i64,
    pub maxflow: i64,
    pub scndry_arc_flow_max: usize,
    pub tile_edge_weight: f64,
    pub max_cycle_fraction: f64,
    pub infile_format: FileFormat,
    pub outfile_format: FileFormat,
    pub corrfile_format: FileFormat,
    pub ampfile_format: FileFormat,
    pub magfile_format: FileFormat,
    pub unwrapped_infile_format: FileFormat,
}

impl Default for RunConfig {
    fn default() -> Self {
        Self {
            unwrapped: false,
            regrow_conn_comps: false,
            eval: false,
            init_only: false,
            init_method: InitMethod::Mst,
            cost_mode: CostMode::Topo,
            amplitude: true,
            verbose: false,
            bperp: 0.0,
            p: -99.999,
            onetilereopt: false,
            rm_tmp_tile: true,
            rm_tile_init: true,
            dump_all: false,
            ntilerow: 1,
            ntilecol: 1,
            rowovrlp: 0,
            colovrlp: 0,
            piecefirstrow: 1,
            piecefirstcol: 1,
            piecenrow: 0,
            piecencol: 0,
            nthreads: 1,
            assemble_only: false,
            tiledir: String::new(),
            parent_pid: i64::from(std::process::id()),
            earthradius: 6_378_000.0,
            altitude: 0.0,
            orbitradius: 7_153_000.0,
            baseline_angle: 1.25 * std::f64::consts::PI,
            transmit_mode: TransmitMode::PingPong,
            baseline: 150.0,
            ncorrlooks: 23.8,
            nearrange: 831_000.0,
            dr: 8.0,
            da: 20.0,
            range_resolution: 10.0,
            lambda: 0.056_564_7,
            kds: 0.02,
            specular_exponent: 8.0,
            dzrcrit_factor: 2.0,
            shadow: false,
            dzeimin: -4.0,
            laywidth: 16,
            layminei: 1.25,
            sloperatio_factor: 1.18,
            sigsqei: 100.0,
            drho: 0.005,
            threshold: 0.001,
            initdzr: 2048.0,
            initdzstep: 100.0,
            dnomincangle: 0.01,
            costscaleambight: 80.0,
            rhosconst1: 1.3,
            rhosconst2: 0.14,
            cstd1: 0.4,
            cstd2: 0.35,
            cstd3: 0.06,
            defaultcorr: 0.01,
            rhominfactor: 1.3,
            dzlaypeak: -2.0,
            azdzfactor: 0.99,
            dzeifactor: 4.0,
            dzeiweight: 0.5,
            dzlayfactor: 1.0,
            layconst: 0.9,
            defothreshfactor: 1.2,
            defomax: 1.2,
            sigsqcorr: 0.05,
            costscale: 100.0,
            defolayconst: 0.9,
            sigsqshortmin: 1,
            sigsqlayfactor: 0.1,
            kperpdpsi: 7,
            kpardpsi: 7,
            krowei: 65,
            kcolei: 257,
            maxcost: 1000,
            layfalloffconst: 2,
            nshortcycle: 200,
            maxflow: 4,
            scndry_arc_flow_max: 8,
            tile_edge_weight: 2.5,
            max_cycle_fraction: 1.0e-5,
            infile_format: FileFormat::ComplexData,
            outfile_format: FileFormat::AltLineData,
            corrfile_format: FileFormat::AltSampleData,
            ampfile_format: FileFormat::AltSampleData,
            magfile_format: FileFormat::FloatData,
            unwrapped_infile_format: FileFormat::AltLineData,
        }
    }
}

/// Reset all CLI/runtime structs to their SNAPHU defaults.
///
/// This is the idiomatic Rust equivalent of C `SetDefaults()`.
pub fn set_defaults(infiles: &mut InputFiles, outfiles: &mut OutputFiles, params: &mut RunConfig) {
    *infiles = InputFiles::default();
    *outfiles = OutputFiles::default();
    *params = RunConfig::default();
}

fn parse_file_format(value: &str) -> Option<FileFormat> {
    match value {
        "COMPLEX_DATA" => Some(FileFormat::ComplexData),
        "FLOAT_DATA" => Some(FileFormat::FloatData),
        "ALT_SAMPLE_DATA" => Some(FileFormat::AltSampleData),
        "ALT_LINE_DATA" => Some(FileFormat::AltLineData),
        "FLOAT_DATA_PHASE_FORMAT" => Some(FileFormat::FloatDataPhase),
        _ => None,
    }
}

/// Apply parsed config-file entries to the runtime structs.
///
/// Maps SNAPHU config keys (e.g. `STATCOSTMODE`, `INFILEFORMAT`) to the
/// corresponding Rust struct fields. Unknown keys are silently ignored for
/// forward compatibility.
pub fn apply_config_entries(
    entries: &[ConfigEntry],
    infiles: &mut InputFiles,
    outfiles: &mut OutputFiles,
    params: &mut RunConfig,
) {
    for entry in entries {
        match entry.key.as_str() {
            // Cost / algorithm mode
            "STATCOSTMODE" => match entry.value.as_str() {
                "TOPO" => params.cost_mode = CostMode::Topo,
                "DEFO" => params.cost_mode = CostMode::Defo,
                "SMOOTH" => params.cost_mode = CostMode::Smooth,
                "NOSTATCOSTS" => params.cost_mode = CostMode::NoStatCosts,
                _ => {}
            },
            "INITMETHOD" => match entry.value.as_str() {
                "MST" => params.init_method = InitMethod::Mst,
                "MCF" => params.init_method = InitMethod::Mcf,
                _ => {}
            },

            // Boolean flags
            "VERBOSE" => params.verbose = is_true(&entry.value),
            "INITONLY" => params.init_only = is_true(&entry.value),
            "UNWRAPPED_IN" => params.unwrapped = is_true(&entry.value),
            "DEBUG" | "DUMPALL" => params.dump_all = is_true(&entry.value),

            // Input files
            "CORRFILE" => infiles.corrfile = entry.value.clone(),
            "AMPFILE" => infiles.ampfile = entry.value.clone(),
            "MAGFILE" => infiles.magfile = entry.value.clone(),
            "ESTIMATEFILE" => infiles.estfile = entry.value.clone(),
            "WEIGHTFILE" => infiles.weightfile = entry.value.clone(),
            "COSTINFILE" => infiles.costinfile = entry.value.clone(),
            "BYTEMASKFILE" => infiles.bytemaskfile = entry.value.clone(),
            "DOTILEMASKFILE" => infiles.dotilemaskfile = entry.value.clone(),
            "INFILE" => infiles.infile = entry.value.clone(),

            // Output files
            "INITFILE" => outfiles.initfile = entry.value.clone(),
            "FLOWFILE" => outfiles.flowfile = entry.value.clone(),
            "EIFILE" => outfiles.eifile = entry.value.clone(),
            "ROWCOSTFILE" => outfiles.rowcostfile = entry.value.clone(),
            "COLCOSTFILE" => outfiles.colcostfile = entry.value.clone(),
            "MSTROWCOSTFILE" => outfiles.mstrowcostfile = entry.value.clone(),
            "MSTCOLCOSTFILE" => outfiles.mstcolcostfile = entry.value.clone(),
            "MSTCOSTSFILE" => outfiles.mstcostsfile = entry.value.clone(),
            "CORRDUMPFILE" => outfiles.corrdumpfile = entry.value.clone(),
            "RAWCORRDUMPFILE" => outfiles.rawcorrdumpfile = entry.value.clone(),
            "OUTFILE" => outfiles.outfile = entry.value.clone(),
            "LOGFILE" => outfiles.logfile = entry.value.clone(),
            "COSTOUTFILE" => outfiles.costoutfile = entry.value.clone(),
            "CONNCOMPFILE" => outfiles.conncompfile = entry.value.clone(),

            // File formats
            "INFILEFORMAT" => {
                if let Some(f) = parse_file_format(&entry.value) {
                    params.infile_format = f;
                }
            }
            "UNWRAPPEDINFILEFORMAT" => {
                if let Some(f) = parse_file_format(&entry.value) {
                    params.unwrapped_infile_format = f;
                }
            }
            "OUTFILEFORMAT" => {
                if let Some(f) = parse_file_format(&entry.value) {
                    params.outfile_format = f;
                }
            }
            "CORRFILEFORMAT" => {
                if let Some(f) = parse_file_format(&entry.value) {
                    params.corrfile_format = f;
                }
            }
            "AMPFILEFORMAT" => {
                if let Some(f) = parse_file_format(&entry.value) {
                    params.ampfile_format = f;
                }
            }
            "MAGFILEFORMAT" => {
                if let Some(f) = parse_file_format(&entry.value) {
                    params.magfile_format = f;
                }
            }

            // Geometry / SAR parameters
            "ORBITRADIUS" => {
                if let Some(v) = string_to_double(&entry.value) {
                    params.orbitradius = v;
                }
            }
            "EARTHRADIUS" => {
                if let Some(v) = string_to_double(&entry.value) {
                    params.earthradius = v;
                }
            }
            "BASELINE" => {
                if let Some(v) = string_to_double(&entry.value) {
                    params.baseline = v;
                }
            }
            "BASELINEANGLE" => {
                if let Some(v) = string_to_double(&entry.value) {
                    params.baseline_angle = v;
                }
            }
            "TRANSMITMODE" => match entry.value.as_str() {
                "PINGPONG" => params.transmit_mode = TransmitMode::PingPong,
                "SINGLEANTTRANSMIT" => params.transmit_mode = TransmitMode::SingleAntenna,
                _ => {}
            },
            "NEARRANGE" => {
                if let Some(v) = string_to_double(&entry.value) {
                    params.nearrange = v;
                }
            }
            "DR" => {
                if let Some(v) = string_to_double(&entry.value) {
                    params.dr = v;
                }
            }
            "DA" => {
                if let Some(v) = string_to_double(&entry.value) {
                    params.da = v;
                }
            }
            "LAMBDA" => {
                if let Some(v) = string_to_double(&entry.value) {
                    params.lambda = v;
                }
            }
            "RANGERES" => {
                if let Some(v) = string_to_double(&entry.value) {
                    params.range_resolution = v;
                }
            }
            "KDS" => {
                if let Some(v) = string_to_double(&entry.value) {
                    params.kds = v;
                }
            }
            "SPECULAREXP" => {
                if let Some(v) = string_to_double(&entry.value) {
                    params.specular_exponent = v;
                }
            }
            "DZRCRITFACTOR" => {
                if let Some(v) = string_to_double(&entry.value) {
                    params.dzrcrit_factor = v;
                }
            }
            "SHADOW" => params.shadow = is_true(&entry.value),
            "DZEIMIN" => {
                if let Some(v) = string_to_double(&entry.value) {
                    params.dzeimin = v;
                }
            }
            "LAYWIDTH" => {
                if let Some(v) = string_to_long(&entry.value) {
                    params.laywidth = v.max(0) as usize;
                }
            }
            "LAYMINEI" => {
                if let Some(v) = string_to_double(&entry.value) {
                    params.layminei = v;
                }
            }
            "SLOPERATIOFACTOR" => {
                if let Some(v) = string_to_double(&entry.value) {
                    params.sloperatio_factor = v;
                }
            }
            "SIGSQEI" => {
                if let Some(v) = string_to_double(&entry.value) {
                    params.sigsqei = v;
                }
            }
            "DRHO" => {
                if let Some(v) = string_to_double(&entry.value) {
                    params.drho = v;
                }
            }
            "THRESHOLD" => {
                if let Some(v) = string_to_double(&entry.value) {
                    params.threshold = v;
                }
            }
            "INITDZR" => {
                if let Some(v) = string_to_double(&entry.value) {
                    params.initdzr = v;
                }
            }
            "INITDZSTEP" => {
                if let Some(v) = string_to_double(&entry.value) {
                    params.initdzstep = v;
                }
            }
            "DNOMINCANGLE" => {
                if let Some(v) = string_to_double(&entry.value) {
                    params.dnomincangle = v;
                }
            }
            "COSTSCALEAMBIGHT" => {
                if let Some(v) = string_to_double(&entry.value) {
                    params.costscaleambight = v;
                }
            }
            "RHOSCONST1" => {
                if let Some(v) = string_to_double(&entry.value) {
                    params.rhosconst1 = v;
                }
            }
            "RHOSCONST2" => {
                if let Some(v) = string_to_double(&entry.value) {
                    params.rhosconst2 = v;
                }
            }
            "CSTD1" => {
                if let Some(v) = string_to_double(&entry.value) {
                    params.cstd1 = v;
                }
            }
            "CSTD2" => {
                if let Some(v) = string_to_double(&entry.value) {
                    params.cstd2 = v;
                }
            }
            "CSTD3" => {
                if let Some(v) = string_to_double(&entry.value) {
                    params.cstd3 = v;
                }
            }
            "DEFAULTCORR" => {
                if let Some(v) = string_to_double(&entry.value) {
                    params.defaultcorr = v;
                }
            }
            "RHOMINFACTOR" => {
                if let Some(v) = string_to_double(&entry.value) {
                    params.rhominfactor = v;
                }
            }
            "DZLAYPEAK" => {
                if let Some(v) = string_to_double(&entry.value) {
                    params.dzlaypeak = v;
                }
            }
            "AZDZFACTOR" => {
                if let Some(v) = string_to_double(&entry.value) {
                    params.azdzfactor = v;
                }
            }
            "DZEIFACTOR" => {
                if let Some(v) = string_to_double(&entry.value) {
                    params.dzeifactor = v;
                }
            }
            "DZEIWEIGHT" => {
                if let Some(v) = string_to_double(&entry.value) {
                    params.dzeiweight = v;
                }
            }
            "DZLAYFACTOR" => {
                if let Some(v) = string_to_double(&entry.value) {
                    params.dzlayfactor = v;
                }
            }
            "LAYCONST" => {
                if let Some(v) = string_to_double(&entry.value) {
                    params.layconst = v;
                }
            }
            "DEFOTHRESHFACTOR" => {
                if let Some(v) = string_to_double(&entry.value) {
                    params.defothreshfactor = v;
                }
            }
            "DEFOMAX_CYCLE" => {
                if let Some(v) = string_to_double(&entry.value) {
                    params.defomax = v;
                }
            }
            "SIGSQCORR" => {
                if let Some(v) = string_to_double(&entry.value) {
                    params.sigsqcorr = v;
                }
            }
            "COSTSCALE" => {
                if let Some(v) = string_to_double(&entry.value) {
                    params.costscale = v;
                }
            }
            "DEFOLAYCONST" => {
                if let Some(v) = string_to_double(&entry.value) {
                    params.defolayconst = v;
                }
            }
            "SIGSQSHORTMIN" => {
                if let Some(v) = string_to_long(&entry.value) {
                    params.sigsqshortmin = v;
                }
            }
            "SIGSQLAYFACTOR" => {
                if let Some(v) = string_to_double(&entry.value) {
                    params.sigsqlayfactor = v;
                }
            }
            "KPERPDPSI" => {
                if let Some(v) = string_to_long(&entry.value) {
                    params.kperpdpsi = v as usize;
                }
            }
            "KPARDPSI" => {
                if let Some(v) = string_to_long(&entry.value) {
                    params.kpardpsi = v as usize;
                }
            }
            "KROWEI" => {
                if let Some(v) = string_to_long(&entry.value) {
                    params.krowei = v.max(0) as usize;
                }
            }
            "KCOLEI" => {
                if let Some(v) = string_to_long(&entry.value) {
                    params.kcolei = v.max(0) as usize;
                }
            }
            "MAXCOST" => {
                if let Some(v) = string_to_long(&entry.value) {
                    params.maxcost = v;
                }
            }
            "LAYFALLOFFCONST" => {
                if let Some(v) = string_to_long(&entry.value) {
                    params.layfalloffconst = v;
                }
            }
            "NCORRLOOKS" => {
                if let Some(v) = string_to_double(&entry.value) {
                    params.ncorrlooks = v;
                }
            }

            // Tile control
            "NTILEROW" => {
                if let Some(v) = string_to_long(&entry.value) {
                    params.ntilerow = v as usize;
                }
            }
            "NTILECOL" => {
                if let Some(v) = string_to_long(&entry.value) {
                    params.ntilecol = v as usize;
                }
            }
            "ROWOVRLP" => {
                if let Some(v) = string_to_long(&entry.value) {
                    params.rowovrlp = v as usize;
                }
            }
            "COLOVRLP" => {
                if let Some(v) = string_to_long(&entry.value) {
                    params.colovrlp = v as usize;
                }
            }
            "NPROC" => {
                if let Some(v) = string_to_long(&entry.value) {
                    params.nthreads = v as usize;
                }
            }
            "SCNDRYARCFLOWMAX" => {
                if let Some(v) = string_to_long(&entry.value) {
                    params.scndry_arc_flow_max = v as usize;
                }
            }
            "TILEEDGEWEIGHT" => {
                if let Some(v) = string_to_double(&entry.value) {
                    params.tile_edge_weight = v;
                }
            }
            "MAXCYCLEFRACTION" => {
                if let Some(v) = string_to_double(&entry.value) {
                    params.max_cycle_fraction = v;
                }
            }

            _ => {} // silently ignore unknown keys
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CheckParamsError {
    EmptyOutputFile,
    ContradictoryModes(&'static str),
    InvalidPositiveParam(&'static str),
    InvalidRange(&'static str),
    InvalidTileGrid,
    InvalidThreads,
    InvalidInputDimensions,
}

/// Validate runtime parameters before unwrapping starts.
///
/// This is the typed Rust equivalent of C `CheckParams()`.
pub fn check_params(
    infiles: &InputFiles,
    outfiles: &OutputFiles,
    linelen: usize,
    nlines: usize,
    params: &RunConfig,
) -> Result<(), CheckParamsError> {
    if outfiles.outfile.is_empty() {
        return Err(CheckParamsError::EmptyOutputFile);
    }
    if linelen == 0 || nlines == 0 {
        return Err(CheckParamsError::InvalidInputDimensions);
    }
    if params.init_only && params.unwrapped {
        return Err(CheckParamsError::ContradictoryModes(
            "init-only with unwrapped input",
        ));
    }
    if params.init_only && params.p >= 0.0 {
        return Err(CheckParamsError::ContradictoryModes(
            "init-only with Lp costs",
        ));
    }
    if matches!(params.cost_mode, CostMode::NoStatCosts) && !(params.init_only || params.p >= 0.0) {
        return Err(CheckParamsError::ContradictoryModes(
            "no-statistical-costs without init-only or Lp mode",
        ));
    }
    if matches!(params.cost_mode, CostMode::NoStatCosts) && !infiles.costinfile.is_empty() {
        return Err(CheckParamsError::ContradictoryModes(
            "no-statistical-costs with input cost file",
        ));
    }
    if matches!(params.cost_mode, CostMode::NoStatCosts) && !outfiles.costoutfile.is_empty() {
        return Err(CheckParamsError::ContradictoryModes(
            "no-statistical-costs with output cost file",
        ));
    }
    if params.earthradius <= 0.0 {
        return Err(CheckParamsError::InvalidPositiveParam("earthradius"));
    }
    if params.altitude > 0.0 {
        if params.earthradius + params.altitude <= params.earthradius {
            return Err(CheckParamsError::InvalidRange("altitude"));
        }
    } else if params.orbitradius < params.earthradius {
        return Err(CheckParamsError::InvalidRange("orbitradius"));
    }
    if matches!(params.cost_mode, CostMode::Topo) && params.baseline < 0.0 {
        return Err(CheckParamsError::InvalidRange("baseline"));
    }
    if params.ncorrlooks <= 0.0 {
        return Err(CheckParamsError::InvalidPositiveParam("ncorrlooks"));
    }
    if params.nearrange <= 0.0 {
        return Err(CheckParamsError::InvalidPositiveParam("nearrange"));
    }
    if params.dr <= 0.0 || params.da <= 0.0 {
        return Err(CheckParamsError::InvalidPositiveParam("dr/da"));
    }
    if params.range_resolution <= 0.0 {
        return Err(CheckParamsError::InvalidPositiveParam("range_resolution"));
    }
    if params.lambda <= 0.0 {
        return Err(CheckParamsError::InvalidPositiveParam("lambda"));
    }
    if !params.kds.is_finite() || params.kds <= 0.0 {
        return Err(CheckParamsError::InvalidPositiveParam("kds"));
    }
    if !params.specular_exponent.is_finite() || params.specular_exponent <= 0.0 {
        return Err(CheckParamsError::InvalidPositiveParam("specular_exponent"));
    }
    if !params.dzrcrit_factor.is_finite() || params.dzrcrit_factor <= 0.0 {
        return Err(CheckParamsError::InvalidPositiveParam("dzrcrit_factor"));
    }
    if params.laywidth == 0 {
        return Err(CheckParamsError::InvalidPositiveParam("laywidth"));
    }
    if !params.sloperatio_factor.is_finite() || params.sloperatio_factor <= 0.0 {
        return Err(CheckParamsError::InvalidPositiveParam("sloperatio_factor"));
    }
    if !params.sigsqei.is_finite() || params.sigsqei < 0.0 {
        return Err(CheckParamsError::InvalidRange("sigsqei"));
    }
    if !params.drho.is_finite() || params.drho <= 0.0 {
        return Err(CheckParamsError::InvalidPositiveParam("drho"));
    }
    if !params.threshold.is_finite() || params.threshold <= 0.0 {
        return Err(CheckParamsError::InvalidPositiveParam("threshold"));
    }
    if !params.initdzr.is_finite() || params.initdzr <= 0.0 {
        return Err(CheckParamsError::InvalidPositiveParam("initdzr"));
    }
    if !params.initdzstep.is_finite() || params.initdzstep <= 0.0 {
        return Err(CheckParamsError::InvalidPositiveParam("initdzstep"));
    }
    if !params.dnomincangle.is_finite() || params.dnomincangle <= 0.0 {
        return Err(CheckParamsError::InvalidPositiveParam("dnomincangle"));
    }
    if !params.costscaleambight.is_finite() || params.costscaleambight <= 0.0 {
        return Err(CheckParamsError::InvalidPositiveParam("costscaleambight"));
    }
    if !params.defaultcorr.is_finite() || !(0.0..=1.0).contains(&params.defaultcorr) {
        return Err(CheckParamsError::InvalidRange("defaultcorr"));
    }
    if !params.rhominfactor.is_finite() || params.rhominfactor <= 0.0 {
        return Err(CheckParamsError::InvalidPositiveParam("rhominfactor"));
    }
    if !params.azdzfactor.is_finite() || params.azdzfactor <= 0.0 {
        return Err(CheckParamsError::InvalidPositiveParam("azdzfactor"));
    }
    if !params.dzeifactor.is_finite() || params.dzeifactor <= 0.0 {
        return Err(CheckParamsError::InvalidPositiveParam("dzeifactor"));
    }
    if !params.dzeiweight.is_finite() || params.dzeiweight < 0.0 {
        return Err(CheckParamsError::InvalidRange("dzeiweight"));
    }
    if !params.dzlayfactor.is_finite() || params.dzlayfactor <= 0.0 {
        return Err(CheckParamsError::InvalidPositiveParam("dzlayfactor"));
    }
    if !params.layconst.is_finite() || params.layconst <= 0.0 || params.layconst >= 1.0 {
        return Err(CheckParamsError::InvalidRange("layconst"));
    }
    if params.maxcost <= 0 {
        return Err(CheckParamsError::InvalidPositiveParam("maxcost"));
    }
    if params.sigsqshortmin <= 0 {
        return Err(CheckParamsError::InvalidPositiveParam("sigsqshortmin"));
    }
    if !params.sigsqlayfactor.is_finite() || params.sigsqlayfactor < 0.0 {
        return Err(CheckParamsError::InvalidRange("sigsqlayfactor"));
    }
    if !params.defothreshfactor.is_finite() || params.defothreshfactor <= 0.0 {
        return Err(CheckParamsError::InvalidPositiveParam("defothreshfactor"));
    }
    if !params.defomax.is_finite() || params.defomax <= 0.0 {
        return Err(CheckParamsError::InvalidPositiveParam("defomax"));
    }
    if !params.sigsqcorr.is_finite() || params.sigsqcorr < 0.0 {
        return Err(CheckParamsError::InvalidRange("sigsqcorr"));
    }
    if !params.costscale.is_finite() || params.costscale <= 0.0 {
        return Err(CheckParamsError::InvalidPositiveParam("costscale"));
    }
    if !params.defolayconst.is_finite() || params.defolayconst <= 0.0 || params.defolayconst >= 1.0
    {
        return Err(CheckParamsError::InvalidRange("defolayconst"));
    }
    if params.kperpdpsi == 0
        || params.kpardpsi == 0
        || params.kperpdpsi.is_multiple_of(2)
        || params.kpardpsi.is_multiple_of(2)
    {
        return Err(CheckParamsError::InvalidRange("kperpdpsi/kpardpsi"));
    }
    if params.krowei == 0 || params.kcolei == 0 {
        return Err(CheckParamsError::InvalidPositiveParam("krowei/kcolei"));
    }
    if params.layfalloffconst <= 0 {
        return Err(CheckParamsError::InvalidPositiveParam("layfalloffconst"));
    }
    if params.nshortcycle < 1 || params.nshortcycle > 8192 {
        return Err(CheckParamsError::InvalidRange("nshortcycle"));
    }
    if params.maxflow <= 0 {
        return Err(CheckParamsError::InvalidPositiveParam("maxflow"));
    }
    if params.maxflow.saturating_mul(params.nshortcycle) > 32_000 {
        return Err(CheckParamsError::InvalidRange("maxflow*nshortcycle"));
    }
    if params.scndry_arc_flow_max == 0 {
        return Err(CheckParamsError::InvalidPositiveParam("scndryarcflowmax"));
    }
    if !params.tile_edge_weight.is_finite() || params.tile_edge_weight <= 0.0 {
        return Err(CheckParamsError::InvalidRange("tileedgeweight"));
    }
    if !params.max_cycle_fraction.is_finite()
        || params.max_cycle_fraction <= 0.0
        || params.max_cycle_fraction > 1.0
    {
        return Err(CheckParamsError::InvalidRange("maxcyclefraction"));
    }
    if params.ntilerow == 0 || params.ntilecol == 0 {
        return Err(CheckParamsError::InvalidTileGrid);
    }
    if params.nthreads == 0 || params.nthreads > 64 {
        return Err(CheckParamsError::InvalidThreads);
    }
    Ok(())
}

/// Returns `true` if the string represents a truthy config value.
///
/// Recognises the same literals as the C `IsTrue()`: `TRUE`, `true`, `True`,
/// `1`, `y`, `Y`, `yes`, `YES`, `Yes`.
pub fn is_true(s: &str) -> bool {
    matches!(
        s,
        "TRUE" | "true" | "True" | "1" | "y" | "Y" | "yes" | "YES" | "Yes"
    )
}

/// Returns `true` if the string represents a falsy config value.
///
/// Recognises the same literals as the C `IsFalse()`: `FALSE`, `false`,
/// `False`, `0`, `n`, `N`, `no`, `NO`, `No`.
pub fn is_false(s: &str) -> bool {
    matches!(
        s,
        "FALSE" | "false" | "False" | "0" | "n" | "N" | "no" | "NO" | "No"
    )
}

/// Uses C-equivalent `strtod` behavior plus SNAPHU's error checks.
///
/// Returns `Some(value)` when the full string is consumed and the parsed value
/// is finite. Returns `None` when parsing fails, extra characters remain, or
/// the parsed value is infinite.
///
/// This intentionally preserves the C quirk that an empty string parses as `0`.
pub fn string_to_double(input: &str) -> Option<f64> {
    if input.is_empty() {
        return Some(0.0);
    }

    // `strtod` accepts leading whitespace. We preserve that behavior while
    // still rejecting any trailing characters.
    let normalized = input.trim_start();
    let value = normalized.parse::<f64>().ok()?;
    if value.is_infinite() {
        None
    } else {
        Some(value)
    }
}

/// Uses C-equivalent `strtol(..., base=10)` behavior plus SNAPHU's checks.
///
/// Returns `Some(value)` when the full string is consumed as base-10 and does
/// not hit the C sentinel checks. Returns `None` when parsing fails, extra
/// characters remain, or when value equals `LONG_MAX`/`LONG_MIN` (matching the
/// legacy function's overflow test).
///
/// This intentionally preserves the C quirk that an empty string parses as `0`.
pub fn string_to_long(input: &str) -> Option<c_long> {
    if input.is_empty() {
        return Some(0);
    }

    // `strtol` accepts leading whitespace. We preserve that behavior while
    // still rejecting any trailing characters.
    let normalized = input.trim_start();
    let value = normalized.parse::<c_long>().ok()?;
    if value == c_long::MAX || value == c_long::MIN {
        None
    } else {
        Some(value)
    }
}

/// Sets a signed-char style boolean (`1` or `0`) from a config token.
///
/// Returns `true` when the input is invalid (matching the C function's
/// "bad parameter" return value), otherwise writes to `boolptr` and
/// returns `false`.
pub fn set_boolean_signed_char(boolptr: &mut i8, input: &str) -> bool {
    if is_true(input) {
        *boolptr = 1;
        false
    } else if is_false(input) {
        *boolptr = 0;
        false
    } else {
        true
    }
}

/// Parsed key/value pair from a configuration line.
///
/// This is the typed Rust equivalent of the tokenized `(str1, str2)` state in
/// C `ParseConfigLine()`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigEntry {
    pub key: String,
    pub value: String,
}

/// Parse one SNAPHU config line into a key/value pair.
///
/// This is the idiomatic Rust equivalent of C `ParseConfigLine()`'s token
/// extraction logic (comment stripping + whitespace tokenization).
/// Returns `Ok(None)` for blank/comment-only lines.
pub fn parse_config_line(line: &str) -> io::Result<Option<ConfigEntry>> {
    let uncommented = line.split('#').next().unwrap_or_default().trim();
    if uncommented.is_empty() {
        return Ok(None);
    }

    let mut parts = uncommented.split_whitespace();
    let Some(key) = parts.next() else {
        return Ok(None);
    };
    let Some(value) = parts.next() else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("missing value for parameter {key}"),
        ));
    };
    if parts.next().is_some() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("too many tokens for parameter {key}"),
        ));
    }

    Ok(Some(ConfigEntry {
        key: key.to_string(),
        value: value.to_string(),
    }))
}

/// Read and parse a SNAPHU-style configuration file.
///
/// This is the idiomatic Rust equivalent of C `ReadConfigFile()`.
pub fn read_config_file(path: &Path) -> io::Result<Vec<ConfigEntry>> {
    let fp = File::open(path)?;
    let reader = BufReader::new(fp);
    let mut entries = Vec::new();
    for (idx, line_res) in reader.lines().enumerate() {
        let line = line_res?;
        match parse_config_line(&line) {
            Ok(Some(entry)) => entries.push(entry),
            Ok(None) => {}
            Err(err) => {
                return Err(io::Error::new(
                    err.kind(),
                    format!("{}:{}: {}", path.display(), idx + 1, err),
                ));
            }
        }
    }
    Ok(entries)
}

/// Metadata written at the top of a config log file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigLogHeader {
    pub program_name: String,
    pub version: String,
    pub hostname: Option<String>,
    pub parent_pid: u32,
    pub cwd: Option<PathBuf>,
    pub command_line: Vec<String>,
}

/// Write runtime parameters to a SNAPHU-compatible config log file.
///
/// This is the idiomatic Rust equivalent of C `WriteConfigLogFile()`.
/// If `logfile` is `None`, no file is written and `Ok(None)` is returned.
pub fn write_config_log_file(
    logfile: Option<&Path>,
    header: &ConfigLogHeader,
    entries: &[ConfigEntry],
) -> io::Result<Option<PathBuf>> {
    let Some(path) = logfile else {
        return Ok(None);
    };

    let mut fp = File::create(path)?;
    writeln!(fp, "# {} v{}", header.program_name, header.version)?;
    if let Some(hostname) = &header.hostname {
        writeln!(fp, "# Host name: {hostname}")?;
    } else {
        writeln!(fp, "# Could not determine host name")?;
    }
    writeln!(fp, "# PID {}", header.parent_pid)?;
    if let Some(cwd) = &header.cwd {
        writeln!(fp, "# Current working directory: {}", cwd.display())?;
    } else {
        writeln!(fp, "# Could not determine current working directory")?;
    }
    write!(fp, "# Command line call:")?;
    for arg in &header.command_line {
        write!(fp, " {arg}")?;
    }
    writeln!(fp)?;
    writeln!(fp)?;

    for entry in entries {
        writeln!(fp, "{}  {}", entry.key, entry.value)?;
    }
    fp.flush()?;
    Ok(Some(path.to_path_buf()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_suffix() -> String {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        format!("{}_{}", std::process::id(), nanos)
    }

    #[test]
    fn is_true_accepts_all_truthy_variants() {
        for s in &["TRUE", "true", "True", "1", "y", "Y", "yes", "YES", "Yes"] {
            assert!(is_true(s), "expected is_true({s:?}) == true");
        }
    }

    #[test]
    fn is_true_rejects_falsy_and_invalid() {
        for s in &["FALSE", "false", "0", "n", "no", "maybe", ""] {
            assert!(!is_true(s), "expected is_true({s:?}) == false");
        }
    }

    #[test]
    fn is_false_accepts_all_falsy_variants() {
        for s in &["FALSE", "false", "False", "0", "n", "N", "no", "NO", "No"] {
            assert!(is_false(s), "expected is_false({s:?}) == true");
        }
    }

    #[test]
    fn is_false_rejects_truthy_and_invalid() {
        for s in &["TRUE", "true", "1", "y", "yes", "maybe", ""] {
            assert!(!is_false(s), "expected is_false({s:?}) == false");
        }
    }

    #[test]
    fn string_to_double_parses_valid_inputs() {
        assert_eq!(string_to_double("1.25"), Some(1.25));
        assert_eq!(string_to_double("  -2.5"), Some(-2.5));
        assert_eq!(string_to_double(""), Some(0.0));
    }

    #[test]
    fn string_to_double_rejects_partial_and_infinite_values() {
        assert_eq!(string_to_double("1.5x"), None);
        assert_eq!(string_to_double("1.5 "), None);
        assert_eq!(string_to_double("abc"), None);
        assert_eq!(string_to_double("1e10000"), None);
    }

    #[test]
    fn string_to_double_accepts_nan_like_c_strtod() {
        let parsed = string_to_double("nan").expect("nan should parse");
        assert!(parsed.is_nan());
    }

    #[test]
    fn string_to_long_parses_valid_inputs() {
        assert_eq!(string_to_long("42"), Some(42));
        assert_eq!(string_to_long("  -17"), Some(-17));
        assert_eq!(string_to_long(""), Some(0));
    }

    #[test]
    fn string_to_long_rejects_partial_invalid_and_extreme_values() {
        assert_eq!(string_to_long("12x"), None);
        assert_eq!(string_to_long("12 "), None);
        assert_eq!(string_to_long("abc"), None);
        assert_eq!(string_to_long(&c_long::MAX.to_string()), None);
        assert_eq!(string_to_long(&c_long::MIN.to_string()), None);
    }

    #[test]
    fn set_boolean_signed_char_sets_truthy_values() {
        let mut value = -1;
        assert!(!set_boolean_signed_char(&mut value, "YES"));
        assert_eq!(value, 1);
    }

    #[test]
    fn set_boolean_signed_char_sets_falsy_values() {
        let mut value = -1;
        assert!(!set_boolean_signed_char(&mut value, "No"));
        assert_eq!(value, 0);
    }

    #[test]
    fn set_boolean_signed_char_rejects_invalid_values_without_mutating() {
        let mut value = 7;
        assert!(set_boolean_signed_char(&mut value, "maybe"));
        assert_eq!(value, 7);
    }

    #[test]
    fn parse_config_line_handles_comments_and_empty_lines() {
        assert_eq!(parse_config_line("   # comment only").unwrap(), None);
        assert_eq!(parse_config_line("").unwrap(), None);

        let parsed = parse_config_line("INFILE  wrapped.bin   # trailing").unwrap();
        assert_eq!(
            parsed,
            Some(ConfigEntry {
                key: "INFILE".to_string(),
                value: "wrapped.bin".to_string()
            })
        );
    }

    #[test]
    fn parse_config_line_rejects_missing_or_extra_tokens() {
        assert!(parse_config_line("INFILE").is_err());
        assert!(parse_config_line("INFILE wrapped.bin extra").is_err());
    }

    #[test]
    fn read_config_file_parses_valid_entries() {
        let path = std::env::temp_dir().join(format!("snaphu_cfg_{}.conf", unique_suffix()));
        fs::write(
            &path,
            "INFILE wrapped.bin\n# comment\nLINELENGTH 1024\nOUTFILE out.bin\n",
        )
        .unwrap();

        let entries = read_config_file(&path).unwrap();
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].key, "INFILE");
        assert_eq!(entries[0].value, "wrapped.bin");
        assert_eq!(entries[1].key, "LINELENGTH");
        assert_eq!(entries[2].key, "OUTFILE");
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn write_config_log_file_writes_header_and_entries() {
        let path = std::env::temp_dir().join(format!("snaphu_cfglog_{}.log", unique_suffix()));
        let header = ConfigLogHeader {
            program_name: "snaphu".to_string(),
            version: "2.0.7".to_string(),
            hostname: Some("host".to_string()),
            parent_pid: 1234,
            cwd: Some(PathBuf::from("/tmp")),
            command_line: vec!["snaphu".to_string(), "wrapped.bin".to_string()],
        };
        let entries = vec![
            ConfigEntry {
                key: "INFILE".to_string(),
                value: "wrapped.bin".to_string(),
            },
            ConfigEntry {
                key: "OUTFILE".to_string(),
                value: "snaphu.out".to_string(),
            },
        ];

        let written = write_config_log_file(Some(&path), &header, &entries)
            .unwrap()
            .unwrap();
        let text = fs::read_to_string(&written).unwrap();
        assert!(text.contains("# snaphu v2.0.7"));
        assert!(text.contains("# PID 1234"));
        assert!(text.contains("INFILE  wrapped.bin"));
        assert!(text.contains("OUTFILE  snaphu.out"));
        fs::remove_file(written).unwrap();
    }

    #[test]
    fn set_defaults_resets_runtime_structs() {
        let mut infiles = InputFiles {
            infile: "in.bin".to_string(),
            ..InputFiles::default()
        };
        let mut outfiles = OutputFiles {
            outfile: "custom.out".to_string(),
            ..OutputFiles::default()
        };
        let mut params = RunConfig {
            verbose: true,
            nthreads: 3,
            ..RunConfig::default()
        };
        set_defaults(&mut infiles, &mut outfiles, &mut params);
        assert_eq!(infiles, InputFiles::default());
        assert_eq!(outfiles, OutputFiles::default());
        assert_eq!(params, RunConfig::default());
    }

    #[test]
    fn check_params_rejects_contradictory_modes() {
        let infiles = InputFiles::default();
        let outfiles = OutputFiles::default();
        let params = RunConfig {
            init_only: true,
            unwrapped: true,
            ..RunConfig::default()
        };
        let err = check_params(&infiles, &outfiles, 128, 64, &params).unwrap_err();
        assert!(matches!(err, CheckParamsError::ContradictoryModes(_)));
    }

    #[test]
    fn check_params_accepts_default_baseline_case() {
        let infiles = InputFiles::default();
        let outfiles = OutputFiles::default();
        let params = RunConfig::default();
        assert!(check_params(&infiles, &outfiles, 128, 64, &params).is_ok());
    }

    #[test]
    fn apply_config_entries_parses_phase_file_format() {
        let entries = vec![
            ConfigEntry {
                key: "INFILEFORMAT".to_string(),
                value: "FLOAT_DATA_PHASE_FORMAT".to_string(),
            },
            ConfigEntry {
                key: "OUTFILEFORMAT".to_string(),
                value: "FLOAT_DATA_PHASE_FORMAT".to_string(),
            },
        ];
        let mut infiles = InputFiles::default();
        let mut outfiles = OutputFiles::default();
        let mut params = RunConfig::default();
        apply_config_entries(&entries, &mut infiles, &mut outfiles, &mut params);
        assert_eq!(params.infile_format, FileFormat::FloatDataPhase);
        assert_eq!(params.outfile_format, FileFormat::FloatDataPhase);
    }

    #[test]
    fn apply_config_entries_sets_dotilemaskfile() {
        let entries = vec![ConfigEntry {
            key: "DOTILEMASKFILE".to_string(),
            value: "tilemask.bin".to_string(),
        }];
        let mut infiles = InputFiles::default();
        let mut outfiles = OutputFiles::default();
        let mut params = RunConfig::default();
        apply_config_entries(&entries, &mut infiles, &mut outfiles, &mut params);
        assert_eq!(infiles.dotilemaskfile, "tilemask.bin");
    }

    #[test]
    fn apply_config_entries_sets_secondary_network_tuning() {
        let entries = vec![
            ConfigEntry {
                key: "SCNDRYARCFLOWMAX".to_string(),
                value: "11".to_string(),
            },
            ConfigEntry {
                key: "TILEEDGEWEIGHT".to_string(),
                value: "3.5".to_string(),
            },
            ConfigEntry {
                key: "MAXCYCLEFRACTION".to_string(),
                value: "0.25".to_string(),
            },
        ];
        let mut infiles = InputFiles::default();
        let mut outfiles = OutputFiles::default();
        let mut params = RunConfig::default();
        apply_config_entries(&entries, &mut infiles, &mut outfiles, &mut params);
        assert_eq!(params.scndry_arc_flow_max, 11);
        assert_eq!(params.tile_edge_weight, 3.5);
        assert_eq!(params.max_cycle_fraction, 0.25);
    }

    #[test]
    fn check_params_rejects_invalid_secondary_network_tuning() {
        let infiles = InputFiles::default();
        let outfiles = OutputFiles::default();

        let bad_flowmax = RunConfig {
            scndry_arc_flow_max: 0,
            ..RunConfig::default()
        };
        assert!(matches!(
            check_params(&infiles, &outfiles, 128, 64, &bad_flowmax),
            Err(CheckParamsError::InvalidPositiveParam("scndryarcflowmax"))
        ));

        let bad_weight = RunConfig {
            tile_edge_weight: 0.0,
            ..RunConfig::default()
        };
        assert!(matches!(
            check_params(&infiles, &outfiles, 128, 64, &bad_weight),
            Err(CheckParamsError::InvalidRange("tileedgeweight"))
        ));

        let bad_cycle = RunConfig {
            max_cycle_fraction: 1.5,
            ..RunConfig::default()
        };
        assert!(matches!(
            check_params(&infiles, &outfiles, 128, 64, &bad_cycle),
            Err(CheckParamsError::InvalidRange("maxcyclefraction"))
        ));
    }
}
