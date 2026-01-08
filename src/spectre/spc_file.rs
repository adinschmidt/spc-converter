//! Complete SPC file extraction including calibration and config.

use crate::parser::{ParseError, StorageObject, unpack_container};
use serde::Serialize;

/// Calibration coefficients for converting pixel index to wavelength.
/// Uses Legendre polynomial expansion: λ(x) = Σ aₖPₖ(x)
/// where x is normalized pixel index (-1 to 1) and Pₖ are Legendre polynomials:
///   P₀(x) = 1
///   P₁(x) = x
///   P₂(x) = ½(3x² - 1)
///   P₃(x) = ½(5x³ - 3x)
#[derive(Debug, Clone, Serialize, Default)]
pub struct Calibration {
    /// Legendre polynomial coefficients [a0, a1, a2, a3]
    pub coefficients: Vec<f64>,
}

impl Calibration {
    /// Convert pixel index (0 to n-1) to wavelength (nm).
    /// Uses Legendre polynomial expansion as defined in the Spectrum Analyzer Suite.
    pub fn pixel_to_wavelength(&self, pixel: usize, num_pixels: usize) -> Option<f64> {
        if self.coefficients.len() != 4 || num_pixels == 0 {
            return None;
        }
        
        // Normalize pixel to -1..1 range: x = 2i/(N-1) - 1
        let x = 2.0 * (pixel as f64) / ((num_pixels - 1) as f64) - 1.0;
        
        // Legendre polynomial evaluation:
        // P₀(x) = 1
        // P₁(x) = x
        // P₂(x) = ½(3x² - 1)
        // P₃(x) = ½(5x³ - 3x)
        let p0 = 1.0;
        let p1 = x;
        let p2 = 0.5 * (3.0 * x * x - 1.0);
        let p3 = 0.5 * (5.0 * x * x * x - 3.0 * x);
        
        let c = &self.coefficients;
        Some(c[0] * p0 + c[1] * p1 + c[2] * p2 + c[3] * p3)
    }
    
    /// Convert pixel index to Raman shift (cm⁻¹) given laser wavelength.
    pub fn pixel_to_raman_shift(&self, pixel: usize, num_pixels: usize, laser_wavelength: f64) -> Option<f64> {
        let wavelength = self.pixel_to_wavelength(pixel, num_pixels)?;
        // Raman shift = 1e7 * (1/λ_laser - 1/λ)
        Some(1e7 * (1.0 / laser_wavelength - 1.0 / wavelength))
    }
    
    /// Generate wavelength axis for all pixels.
    pub fn generate_wavelength_axis(&self, num_pixels: usize) -> Option<Vec<f64>> {
        if self.coefficients.len() != 4 || num_pixels == 0 {
            return None;
        }
        
        let axis: Vec<f64> = (0..num_pixels)
            .map(|i| self.pixel_to_wavelength(i, num_pixels).unwrap())
            .collect();
        
        Some(axis)
    }
    
    /// Generate Raman shift axis for all pixels.
    pub fn generate_raman_shift_axis(&self, num_pixels: usize, laser_wavelength: f64) -> Option<Vec<f64>> {
        if self.coefficients.len() != 4 || num_pixels == 0 {
            return None;
        }
        
        let axis: Vec<f64> = (0..num_pixels)
            .map(|i| self.pixel_to_raman_shift(i, num_pixels, laser_wavelength).unwrap())
            .collect();
        
        Some(axis)
    }
}

/// Axis type enumeration for display preferences.
#[derive(Debug, Clone, Copy, Serialize, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AxisType {
    /// Display as pixel indices
    #[default]
    Pixels = 0,
    /// Display as wavelengths (nm)
    Wavelengths = 1,
    /// Display as Raman shifts (cm⁻¹)
    RamanShifts = 2,
}

impl From<i32> for AxisType {
    fn from(value: i32) -> Self {
        match value {
            1 => AxisType::Wavelengths,
            2 => AxisType::RamanShifts,
            _ => AxisType::Pixels,
        }
    }
}

/// Configuration parameters stored with the spectrum.
#[derive(Debug, Clone, Serialize, Default)]
pub struct Config {
    /// Raman laser wavelength in nm (typically 785, 532, etc.)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raman_wavelength: Option<f64>,
    /// Camera exposure time
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exposure: Option<f64>,
    /// Camera gain
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gain: Option<f64>,
    /// Smoothing kernel size
    #[serde(skip_serializing_if = "Option::is_none")]
    pub smoothing: Option<i32>,
    /// Number of frames to average
    #[serde(skip_serializing_if = "Option::is_none")]
    pub average: Option<i32>,
    /// Savitzky-Golay window size
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sgolay_window: Option<i32>,
    /// Savitzky-Golay polynomial order
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sgolay_order: Option<i32>,
    /// Savitzky-Golay derivative order
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sgolay_deriv: Option<i32>,
    /// Median filter enabled
    #[serde(skip_serializing_if = "Option::is_none")]
    pub medfilt: Option<bool>,
    /// Baseline removal enabled
    #[serde(skip_serializing_if = "Option::is_none")]
    pub baseline: Option<bool>,
    /// Savitzky-Golay filter enabled
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sgolay: Option<bool>,
    /// Preferred axis type for display
    #[serde(skip_serializing_if = "Option::is_none")]
    pub axis: Option<AxisType>,
    /// Any other config values as key-value pairs
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub other: Vec<(String, String)>,
}

/// Complete extracted data from an SPC file.
#[derive(Debug, Clone, Serialize)]
pub struct SpcFile {
    /// Unique identifier for this measurement (typically camera serial number).
    pub uid: String,
    /// Spectral intensity data (Y-axis values).
    pub data: Vec<f64>,
    /// Blank/reference spectrum for calibration.
    pub blank: Vec<f64>,
    /// Calibration data if present.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub calibration: Option<Calibration>,
    /// Configuration parameters if present.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config: Option<Config>,
    /// Generated wavelength axis (if calibration is present).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wavelength_axis: Option<Vec<f64>>,
    /// Generated Raman shift axis (if calibration and raman_wavelength are present).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raman_shift_axis: Option<Vec<f64>>,
}

impl SpcFile {
    /// Parse from raw file bytes (handles container encryption/compression).
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, ParseError> {
        // First unpack the container (decrypt + decompress)
        let buffers = unpack_container(bytes)?;
        
        if buffers.is_empty() {
            return Err(ParseError::MissingField("No buffers in container".to_string()));
        }

        // Parse each buffer as a StorageObject
        let mut data_obj: Option<StorageObject> = None;
        let mut calibration_obj: Option<StorageObject> = None;
        let mut config_obj: Option<StorageObject> = None;
        
        for buffer in &buffers {
            if let Ok(obj) = StorageObject::from_bytes(buffer) {
                match obj.var_name.as_str() {
                    "data" => data_obj = Some(obj),
                    "calibration" => calibration_obj = Some(obj),
                    "config" => config_obj = Some(obj),
                    _ => {} // Ignore unknown objects
                }
            }
        }
        
        // Data object is required
        let data_obj = data_obj.ok_or_else(|| ParseError::MissingField("data".to_string()))?;
        
        // Extract SpectreFile data
        let uid = extract_string_child(&data_obj, "m_uid")?;
        let data = extract_double_vector_child(&data_obj, "m_data")?;
        let blank = extract_double_vector_child(&data_obj, "m_blank")?;
        
        // Extract calibration if present
        let calibration = calibration_obj.and_then(|obj| {
            extract_double_vector(&obj).ok().map(|coefficients| Calibration { coefficients })
        });
        
        // Extract config if present
        let config = config_obj.and_then(|obj| extract_config(&obj).ok());
        
        // Generate axes if possible
        let num_pixels = data.len();
        let wavelength_axis = calibration.as_ref()
            .and_then(|cal| cal.generate_wavelength_axis(num_pixels));
        
        let raman_shift_axis = calibration.as_ref()
            .and_then(|cal| {
                config.as_ref()
                    .and_then(|cfg| cfg.raman_wavelength)
                    .and_then(|laser| cal.generate_raman_shift_axis(num_pixels, laser))
            });
        
        Ok(Self {
            uid,
            data,
            blank,
            calibration,
            config,
            wavelength_axis,
            raman_shift_axis,
        })
    }

    /// Read from a file path.
    pub fn from_file(path: &std::path::Path) -> Result<Self, ParseError> {
        let bytes = std::fs::read(path)?;
        Self::from_bytes(&bytes)
    }
    
    /// Check if this file has calibration data.
    pub fn has_calibration(&self) -> bool {
        self.calibration.is_some()
    }
    
    /// Check if this file has Raman shift data.
    pub fn has_raman_shift(&self) -> bool {
        self.raman_shift_axis.is_some()
    }
}

/// Extract a storage_string child as a String.
fn extract_string_child(obj: &StorageObject, name: &str) -> Result<String, ParseError> {
    let child = obj
        .find_child(name)
        .ok_or_else(|| ParseError::MissingField(name.to_string()))?;

    // storage_string stores: "size" (size_t) and "data" (char array)
    let data_var = child
        .find_var("data")
        .ok_or_else(|| ParseError::MissingField(format!("{}.data", name)))?;

    // Data is null-terminated string bytes
    let end = data_var
        .data
        .iter()
        .position(|&b| b == 0)
        .unwrap_or(data_var.data.len());

    String::from_utf8(data_var.data[..end].to_vec())
        .map_err(|_| ParseError::MissingField(format!("{} (invalid UTF-8)", name)))
}

/// Extract a storage_vector<double> child as Vec<f64>.
fn extract_double_vector_child(obj: &StorageObject, name: &str) -> Result<Vec<f64>, ParseError> {
    let child = obj
        .find_child(name)
        .ok_or_else(|| ParseError::MissingField(name.to_string()))?;

    extract_double_vector(child)
}

/// Extract a storage_vector<double> from a StorageObject.
fn extract_double_vector(obj: &StorageObject) -> Result<Vec<f64>, ParseError> {
    // storage_vector<double> stores each element as a variable with empty name
    let mut values = Vec::with_capacity(obj.variables.len());

    for var in &obj.variables {
        if var.data.len() != 8 {
            return Err(ParseError::TypeMismatch {
                expected: "double (8 bytes)".to_string(),
                actual: format!("{} bytes", var.data.len()),
            });
        }

        let value = f64::from_le_bytes(var.data[..8].try_into().unwrap());
        values.push(value);
    }

    Ok(values)
}

/// Extract config parameters from a StorageObject.
/// The config stores wndParametersDialog fields as child objects (dynamic_var<T>),
/// each containing a "data" variable with the actual value.
fn extract_config(obj: &StorageObject) -> Result<Config, ParseError> {
    let mut config = Config::default();
    
    // The config object contains children for each parameter
    // Each child is a dynamic_var<T> which stores the value in a variable named "data"
    for child in &obj.children {
        // Try to find a "data" variable in the child
        if let Some(data_var) = child.find_var("data") {
            let name = child.var_name.as_str();
            
            if data_var.data.len() == 8 {
                // Double value
                let value = f64::from_le_bytes(data_var.data[..8].try_into().unwrap());
                match name {
                    "raman_wavelength" => config.raman_wavelength = Some(value),
                    "exposure" => config.exposure = Some(value),
                    "gain" => config.gain = Some(value),
                    _ => {
                        // Store as generic double param
                        config.other.push((name.to_string(), format!("{}", value)));
                    }
                }
            } else if data_var.data.len() == 4 {
                // Int32 value
                let value = i32::from_le_bytes(data_var.data[..4].try_into().unwrap());
                match name {
                    "smoothing" => config.smoothing = Some(value),
                    "average" => config.average = Some(value),
                    "sgolay_window" => config.sgolay_window = Some(value),
                    "sgolay_order" => config.sgolay_order = Some(value),
                    "sgolay_deriv" => config.sgolay_deriv = Some(value),
                    "axis" => config.axis = Some(AxisType::from(value)),
                    _ => {
                        config.other.push((name.to_string(), format!("{}", value)));
                    }
                }
            } else if data_var.data.len() == 1 {
                // Bool value (stored as single byte)
                let value = data_var.data[0] != 0;
                match name {
                    "medfilt" => config.medfilt = Some(value),
                    "baseline" => config.baseline = Some(value),
                    "sgolay" => config.sgolay = Some(value),
                    _ => {
                        config.other.push((name.to_string(), format!("{}", value)));
                    }
                }
            }
        }
    }
    
    // Also check variables on the object itself (for simpler storage)
    for var in &obj.variables {
        if var.data.len() == 8 {
            let value = f64::from_le_bytes(var.data[..8].try_into().unwrap());
            if var.name == "raman_wavelength" && config.raman_wavelength.is_none() {
                config.raman_wavelength = Some(value);
            }
        }
    }
    
    Ok(config)
}

