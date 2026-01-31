//! SPC Converter Library
//!
//! Parses Spectrum Analyzer Suite .spc files and converts them to open formats.

#![allow(clippy::multiple_crate_versions)]

pub mod output;
pub mod parser;
pub mod spectre;

pub use parser::StorageObject;
pub use spectre::{Calibration, Config, SpcFile, SpectreFile};
