//! Plot output format - PNG spectrum visualization.

use crate::spectre::SpcFile;
use std::io;
use std::path::Path;

use plotters::prelude::*;
use plotters::backend::BitMapBackend;

/// Axis type selected for plotting, with descriptive information.
#[derive(Debug, Clone)]
pub struct PlotAxisInfo {
    pub name: &'static str,
    pub unit: &'static str,
    pub values: Vec<f64>,
    /// Whether the x-axis should be reversed (high to low, spectroscopy convention)
    pub reversed: bool,
}

/// Determines the best axis to use for plotting based on available data.
/// Priority: Raman Shift > Wavelength > Pixel Index
pub fn select_best_axis(spc: &SpcFile) -> PlotAxisInfo {
    if let Some(ref raman) = spc.raman_shift_axis {
        PlotAxisInfo {
            name: "Raman Shift",
            unit: "cm⁻¹",
            values: raman.clone(),
            reversed: true, // Spectroscopy convention: high to low
        }
    } else if let Some(ref wavelength) = spc.wavelength_axis {
        PlotAxisInfo {
            name: "Wavelength",
            unit: "nm",
            values: wavelength.clone(),
            reversed: false,
        }
    } else {
        PlotAxisInfo {
            name: "Pixel Index",
            unit: "",
            values: (0..spc.data.len()).map(|i| i as f64).collect(),
            reversed: false,
        }
    }
}

/// Generate a PNG plot of the spectrum.
///
/// The plot will intelligently select the best available x-axis:
/// - Raman shift (cm⁻¹) if laser wavelength and calibration are present
/// - Wavelength (nm) if only calibration is present
/// - Pixel index as fallback
///
/// # Arguments
/// * `spc` - The parsed SPC file
/// * `output_path` - Output path for the PNG file
/// * `width` - Image width in pixels (default: 1200)
/// * `height` - Image height in pixels (default: 600)
pub fn write_plot<P: AsRef<Path>>(
    spc: &SpcFile,
    output_path: P,
    width: u32,
    height: u32,
) -> io::Result<()> {
    let axis = select_best_axis(spc);
    
    // Calculate data ranges with padding
    let x_min = axis.values.iter().cloned().fold(f64::INFINITY, f64::min);
    let x_max = axis.values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let y_min = spc.data.iter().cloned().fold(f64::INFINITY, f64::min);
    let y_max = spc.data.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    
    // Add padding to y-axis
    let y_padding = (y_max - y_min) * 0.05;
    let y_min = y_min - y_padding;
    let y_max = y_max + y_padding;
    
    // Build axis label
    let x_label = if axis.unit.is_empty() {
        axis.name.to_string()
    } else {
        format!("{} ({})", axis.name, axis.unit)
    };
    
    // Build title
    let title = if let Some(ref cfg) = spc.config {
        if let Some(laser) = cfg.raman_wavelength {
            format!("Spectrum ({}nm laser)", laser)
        } else {
            "Spectrum".to_string()
        }
    } else {
        "Spectrum".to_string()
    };
    
    // Create the chart
    let root = BitMapBackend::new(output_path.as_ref(), (width, height))
        .into_drawing_area();
    
    root.fill(&WHITE)
        .map_err(|e: DrawingAreaErrorKind<_>| io::Error::new(io::ErrorKind::Other, format!("{:?}", e)))?;
    
    // Build x-axis range (reversed for Raman shift - spectroscopy convention)
    let (x_start, x_end) = if axis.reversed {
        (x_max, x_min)  // High to low
    } else {
        (x_min, x_max)  // Normal: low to high
    };
    
    let mut chart = ChartBuilder::on(&root)
        .caption(&title, ("sans-serif", 24).into_font())
        .margin(20)
        .x_label_area_size(50)
        .y_label_area_size(70)
        .build_cartesian_2d(x_start..x_end, y_min..y_max)
        .map_err(|e: DrawingAreaErrorKind<_>| io::Error::new(io::ErrorKind::Other, format!("{:?}", e)))?;
    
    chart
        .configure_mesh()
        .x_desc(&x_label)
        .y_desc("Intensity")
        .axis_desc_style(("sans-serif", 16))
        .label_style(("sans-serif", 12))
        .draw()
        .map_err(|e: DrawingAreaErrorKind<_>| io::Error::new(io::ErrorKind::Other, format!("{:?}", e)))?;
    
    // Draw the spectrum line
    let data_points: Vec<(f64, f64)> = axis.values
        .iter()
        .zip(spc.data.iter())
        .map(|(&x, &y)| (x, y))
        .collect();
    
    chart
        .draw_series(LineSeries::new(data_points, &BLUE))
        .map_err(|e: DrawingAreaErrorKind<_>| io::Error::new(io::ErrorKind::Other, format!("{:?}", e)))?;
    
    // Render to file
    root.present()
        .map_err(|e: DrawingAreaErrorKind<_>| io::Error::new(io::ErrorKind::Other, format!("{:?}", e)))?;
    
    Ok(())
}

/// Generate a PNG plot with default dimensions (1200x600).
pub fn write_plot_default<P: AsRef<Path>>(spc: &SpcFile, output_path: P) -> io::Result<()> {
    write_plot(spc, output_path, 1200, 600)
}
