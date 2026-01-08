//! SpectreFile extraction from StorageObject tree.

use crate::parser::{ParseError, StorageObject};
use serde::Serialize;

/// Extracted spectral data from an SPC file.
#[derive(Debug, Clone, Serialize)]
pub struct SpectreFile {
    /// Unique identifier for this measurement.
    pub uid: String,
    /// Spectral intensity data (Y-axis values).
    pub data: Vec<f64>,
    /// Blank/reference spectrum for calibration.
    pub blank: Vec<f64>,
}

impl SpectreFile {
    /// Extract SpectreFile from a parsed StorageObject.
    pub fn from_storage_object(obj: &StorageObject) -> Result<Self, ParseError> {
        // Find m_uid child
        let uid = extract_string_child(obj, "m_uid")?;

        // Find m_data child (storage_vector<double>)
        let data = extract_double_vector_child(obj, "m_data")?;

        // Find m_blank child (storage_vector<double>)
        let blank = extract_double_vector_child(obj, "m_blank")?;

        Ok(Self { uid, data, blank })
    }

    /// Parse from raw file bytes (handles container encryption/compression).
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, ParseError> {
        // First unpack the container (decrypt + decompress)
        let buffers = crate::parser::unpack_container(bytes)?;
        
        if buffers.is_empty() {
            return Err(ParseError::MissingField("No buffers in container".to_string()));
        }

        // Parse the first buffer as a StorageObject
        let obj = StorageObject::from_bytes(&buffers[0])?;
        Self::from_storage_object(&obj)
    }

    /// Read from a file path.
    pub fn from_file(path: &std::path::Path) -> Result<Self, ParseError> {
        let bytes = std::fs::read(path)?;
        Self::from_bytes(&bytes)
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

    // storage_vector<double> stores each element as a variable with empty name
    // Type should be "double" and size 8 bytes each
    let mut values = Vec::with_capacity(child.variables.len());

    for var in &child.variables {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spectre_file_structure() {
        // Basic struct tests - actual parsing requires sample files
        let sf = SpectreFile {
            uid: "test".to_string(),
            data: vec![1.0, 2.0, 3.0],
            blank: vec![0.1, 0.2, 0.3],
        };
        assert_eq!(sf.data.len(), 3);
        assert_eq!(sf.blank.len(), 3);
    }
}
