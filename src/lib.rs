//! SPC Converter Library
//!
//! Parses Spectrum Analyzer Suite .spc files and converts them to open formats.

pub mod parser;
pub mod spectre;
pub mod output;

pub use parser::StorageObject;
pub use spectre::{SpectreFile, SpcFile, Calibration, Config};
