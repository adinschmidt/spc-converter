//! Pairs output format - LLM-friendly x,y pairs with minimal context header.

use crate::spectre::SpcFile;
use std::io::{self, Write};

/// Write SpcFile as LLM-friendly pairs format.
///
/// Format:
/// ```text
/// # Raman Spectrum
/// # X-axis: Raman Shift (cm⁻¹), Y-axis: Intensity
/// # Laser: 785nm, Points: 2048
///
/// 176.5, 1024.3
/// 180.2, 1089.7
/// ...
/// ```
pub fn write_pairs<W: Write>(spc: &SpcFile, mut writer: W) -> io::Result<()> {
    // Determine which x-axis to use (prefer Raman shift, then wavelength, then index)
    let (x_axis_name, x_axis_unit, x_values): (&str, &str, Vec<f64>) =
        if let Some(ref raman) = spc.raman_shift_axis {
            ("Raman Shift", "cm⁻¹", raman.clone())
        } else if let Some(ref wavelength) = spc.wavelength_axis {
            ("Wavelength", "nm", wavelength.clone())
        } else {
            ("Pixel Index", "", (0..spc.data.len()).map(|i| i as f64).collect())
        };

    // Write header comments
    writeln!(writer, "# Raman Spectrum")?;
    
    if x_axis_unit.is_empty() {
        writeln!(writer, "# X-axis: {}, Y-axis: Intensity", x_axis_name)?;
    } else {
        writeln!(writer, "# X-axis: {} ({}), Y-axis: Intensity", x_axis_name, x_axis_unit)?;
    }

    // Add laser wavelength if available
    if let Some(ref cfg) = spc.config {
        if let Some(laser) = cfg.raman_wavelength {
            writeln!(writer, "# Laser: {}nm, Points: {}", laser, spc.data.len())?;
        } else {
            writeln!(writer, "# Points: {}", spc.data.len())?;
        }
    } else {
        writeln!(writer, "# Points: {}", spc.data.len())?;
    }

    writeln!(writer)?; // Blank line before data

    // Write x,y pairs
    for (x, y) in x_values.iter().zip(spc.data.iter()) {
        writeln!(writer, "{}, {}", x, y)?;
    }

    Ok(())
}

/// Write SpcFile as pairs format string.
pub fn to_pairs_string(spc: &SpcFile) -> io::Result<String> {
    let mut buf = Vec::new();
    write_pairs(spc, &mut buf)?;
    String::from_utf8(buf).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}
