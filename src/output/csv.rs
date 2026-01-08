//! CSV output format.

use crate::spectre::{SpectreFile, SpcFile};
use std::io::{self, Write};

/// Write SpectreFile as CSV to a writer.
///
/// Format: index,intensity,blank
pub fn write_csv<W: Write>(spectre: &SpectreFile, mut writer: W) -> io::Result<()> {
    // Header
    writeln!(writer, "index,intensity,blank")?;

    // Determine max length (data and blank may differ in length)
    let max_len = spectre.data.len().max(spectre.blank.len());

    for i in 0..max_len {
        let intensity = spectre.data.get(i).copied().unwrap_or(f64::NAN);
        let blank = spectre.blank.get(i).copied().unwrap_or(f64::NAN);
        writeln!(writer, "{},{},{}", i, intensity, blank)?;
    }

    Ok(())
}

/// Write SpectreFile as CSV string.
pub fn to_csv_string(spectre: &SpectreFile) -> io::Result<String> {
    let mut buf = Vec::new();
    write_csv(spectre, &mut buf)?;
    String::from_utf8(buf).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

/// Write SpcFile as CSV to a writer.
///
/// If calibration is present, includes wavelength/wavenumber columns.
/// Format: index,wavelength,raman_shift,intensity,blank
pub fn write_csv_spc<W: Write>(spc: &SpcFile, mut writer: W) -> io::Result<()> {
    // Determine what columns we have
    let has_wavelength = spc.wavelength_axis.is_some();
    let has_raman = spc.raman_shift_axis.is_some();
    
    // Write header
    let mut header = String::from("index");
    if has_wavelength {
        header.push_str(",wavelength_nm");
    }
    if has_raman {
        header.push_str(",raman_shift_cm-1");
    }
    header.push_str(",intensity");
    if !spc.blank.is_empty() {
        header.push_str(",blank");
    }
    writeln!(writer, "{}", header)?;

    // Determine max length
    let max_len = spc.data.len().max(spc.blank.len());
    
    let wavelengths = spc.wavelength_axis.as_ref();
    let raman_shifts = spc.raman_shift_axis.as_ref();

    for i in 0..max_len {
        // Index
        write!(writer, "{}", i)?;
        
        // Wavelength
        if has_wavelength {
            let wl = wavelengths.and_then(|v| v.get(i)).copied().unwrap_or(f64::NAN);
            write!(writer, ",{}", wl)?;
        }
        
        // Raman shift
        if has_raman {
            let rs = raman_shifts.and_then(|v| v.get(i)).copied().unwrap_or(f64::NAN);
            write!(writer, ",{}", rs)?;
        }
        
        // Intensity
        let intensity = spc.data.get(i).copied().unwrap_or(f64::NAN);
        write!(writer, ",{}", intensity)?;
        
        // Blank
        if !spc.blank.is_empty() {
            let blank = spc.blank.get(i).copied().unwrap_or(f64::NAN);
            write!(writer, ",{}", blank)?;
        }
        
        writeln!(writer)?;
    }

    Ok(())
}

/// Write SpcFile as CSV string.
pub fn to_csv_string_spc(spc: &SpcFile) -> io::Result<String> {
    let mut buf = Vec::new();
    write_csv_spc(spc, &mut buf)?;
    String::from_utf8(buf).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}
