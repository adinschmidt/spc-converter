//! Header structures for the binary storage format.

use thiserror::Error;

/// Errors that can occur during parsing.
#[derive(Error, Debug)]
pub enum ParseError {
    #[error("File too small: expected at least {expected} bytes, got {actual}")]
    FileTooSmall { expected: usize, actual: usize },

    #[error("Invalid offset: {offset} exceeds buffer size {size}")]
    InvalidOffset { offset: u64, size: usize },

    #[error("String not null-terminated at offset {0}")]
    UnterminatedString(u64),

    #[error("Variable count mismatch: header says {expected}, section has {actual}")]
    VarCountMismatch { expected: u64, actual: usize },

    #[error("Child count mismatch: header says {expected}, section has {actual}")]
    ChildCountMismatch { expected: u64, actual: usize },

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Missing required field: {0}")]
    MissingField(String),

    #[error("Type mismatch: expected {expected}, got {actual}")]
    TypeMismatch { expected: String, actual: String },
}

/// Buffer section descriptor {offset, size}.
#[derive(Debug, Clone, Copy)]
pub struct BufferSection {
    pub offset: u64,
    pub size: u64,
}

impl BufferSection {
    /// Read from 16 bytes at the given position.
    pub fn from_bytes(data: &[u8]) -> Self {
        Self {
            offset: u64::from_le_bytes(data[0..8].try_into().unwrap()),
            size: u64::from_le_bytes(data[8..16].try_into().unwrap()),
        }
    }
}

/// Main header structure (96 bytes, packed).
#[derive(Debug, Clone)]
pub struct PackHeader {
    pub type_name_offset: u64,
    pub owner_offset: u64,
    pub name_offset: u64,
    pub num_vars: u64,
    pub num_children: u64,
    pub strings: BufferSection,
    pub vars: BufferSection,
    pub children: BufferSection,
    pub data: BufferSection,
}

impl PackHeader {
    pub const SIZE: usize = 104; // 40 bytes + 4×16 bytes for buffer sections

    /// Parse header from bytes.
    pub fn from_bytes(data: &[u8]) -> Result<Self, ParseError> {
        if data.len() < Self::SIZE {
            return Err(ParseError::FileTooSmall {
                expected: Self::SIZE,
                actual: data.len(),
            });
        }

        Ok(Self {
            type_name_offset: u64::from_le_bytes(data[0..8].try_into().unwrap()),
            owner_offset: u64::from_le_bytes(data[8..16].try_into().unwrap()),
            name_offset: u64::from_le_bytes(data[16..24].try_into().unwrap()),
            num_vars: u64::from_le_bytes(data[24..32].try_into().unwrap()),
            num_children: u64::from_le_bytes(data[32..40].try_into().unwrap()),
            strings: BufferSection::from_bytes(&data[40..56]),
            vars: BufferSection::from_bytes(&data[56..72]),
            children: BufferSection::from_bytes(&data[72..88]),
            data: BufferSection::from_bytes(&data[88..104]),
        })
    }
}

/// Variable descriptor (40 bytes, packed).
#[derive(Debug, Clone)]
pub struct PackVar {
    pub owner_offset: u64,
    pub name_offset: u64,
    pub type_offset: u64,
    pub data_offset: u64,
    pub bytes_size: u64,
}

impl PackVar {
    pub const SIZE: usize = 40;

    /// Parse from bytes.
    pub fn from_bytes(data: &[u8]) -> Self {
        Self {
            owner_offset: u64::from_le_bytes(data[0..8].try_into().unwrap()),
            name_offset: u64::from_le_bytes(data[8..16].try_into().unwrap()),
            type_offset: u64::from_le_bytes(data[16..24].try_into().unwrap()),
            data_offset: u64::from_le_bytes(data[24..32].try_into().unwrap()),
            bytes_size: u64::from_le_bytes(data[32..40].try_into().unwrap()),
        }
    }
}

/// Child object descriptor (32 bytes, packed).
#[derive(Debug, Clone)]
pub struct PackChild {
    pub owner_offset: u64,
    pub name_offset: u64,
    pub data_offset: u64,
    pub size: u64,
}

impl PackChild {
    pub const SIZE: usize = 32;

    /// Parse from bytes.
    pub fn from_bytes(data: &[u8]) -> Self {
        Self {
            owner_offset: u64::from_le_bytes(data[0..8].try_into().unwrap()),
            name_offset: u64::from_le_bytes(data[8..16].try_into().unwrap()),
            data_offset: u64::from_le_bytes(data[16..24].try_into().unwrap()),
            size: u64::from_le_bytes(data[24..32].try_into().unwrap()),
        }
    }
}
