//! JSON output format.

use crate::spectre::{SpectreFile, SpcFile};
use serde::Serialize;
use std::io::Write;

/// JSON output with metadata (legacy SpectreFile format).
#[derive(Serialize)]
pub struct JsonOutput<'a> {
    pub uid: &'a str,
    pub data: &'a [f64],
    pub blank: &'a [f64],
    pub metadata: JsonMetadata,
}

#[derive(Serialize)]
pub struct JsonMetadata {
    pub point_count: usize,
    pub blank_count: usize,
    pub source_format: &'static str,
}

/// Write SpectreFile as JSON to a writer.
pub fn write_json<W: Write>(
    spectre: &SpectreFile,
    writer: W,
    pretty: bool,
) -> Result<(), serde_json::Error> {
    let output = JsonOutput {
        uid: &spectre.uid,
        data: &spectre.data,
        blank: &spectre.blank,
        metadata: JsonMetadata {
            point_count: spectre.data.len(),
            blank_count: spectre.blank.len(),
            source_format: "pulsar_spc_v1",
        },
    };

    if pretty {
        serde_json::to_writer_pretty(writer, &output)
    } else {
        serde_json::to_writer(writer, &output)
    }
}

/// Write SpectreFile as JSON string.
pub fn to_json_string(spectre: &SpectreFile, pretty: bool) -> Result<String, serde_json::Error> {
    let output = JsonOutput {
        uid: &spectre.uid,
        data: &spectre.data,
        blank: &spectre.blank,
        metadata: JsonMetadata {
            point_count: spectre.data.len(),
            blank_count: spectre.blank.len(),
            source_format: "pulsar_spc_v1",
        },
    };

    if pretty {
        serde_json::to_string_pretty(&output)
    } else {
        serde_json::to_string(&output)
    }
}

/// Write SpcFile (with calibration) as JSON to a writer.
pub fn write_json_spc<W: Write>(
    spc: &SpcFile,
    writer: W,
    pretty: bool,
) -> Result<(), serde_json::Error> {
    // SpcFile is already Serialize, so we output it directly
    if pretty {
        serde_json::to_writer_pretty(writer, spc)
    } else {
        serde_json::to_writer(writer, spc)
    }
}

/// Write SpcFile as JSON string.
pub fn to_json_string_spc(spc: &SpcFile, pretty: bool) -> Result<String, serde_json::Error> {
    if pretty {
        serde_json::to_string_pretty(spc)
    } else {
        serde_json::to_string(spc)
    }
}
