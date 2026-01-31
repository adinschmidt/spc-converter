//! Header structures for the binary storage format.

use thiserror::Error;

use super::read_u64_le;

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
    pub const SIZE: usize = 16;

    /// Read from 16 bytes at the given position.
    ///
    /// # Errors
    /// Returns [`ParseError::FileTooSmall`] if `data` is shorter than [`Self::SIZE`].
    pub fn from_bytes(data: &[u8]) -> Result<Self, ParseError> {
        if data.len() < Self::SIZE {
            return Err(ParseError::FileTooSmall {
                expected: Self::SIZE,
                actual: data.len(),
            });
        }

        Ok(Self {
            offset: read_u64_le(data, 0)?,
            size: read_u64_le(data, 8)?,
        })
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
    ///
    /// # Errors
    /// Returns [`ParseError::FileTooSmall`] if `data` is shorter than [`Self::SIZE`].
    pub fn from_bytes(data: &[u8]) -> Result<Self, ParseError> {
        if data.len() < Self::SIZE {
            return Err(ParseError::FileTooSmall {
                expected: Self::SIZE,
                actual: data.len(),
            });
        }

        Ok(Self {
            type_name_offset: read_u64_le(data, 0)?,
            owner_offset: read_u64_le(data, 8)?,
            name_offset: read_u64_le(data, 16)?,
            num_vars: read_u64_le(data, 24)?,
            num_children: read_u64_le(data, 32)?,
            strings: BufferSection {
                offset: read_u64_le(data, 40)?,
                size: read_u64_le(data, 48)?,
            },
            vars: BufferSection {
                offset: read_u64_le(data, 56)?,
                size: read_u64_le(data, 64)?,
            },
            children: BufferSection {
                offset: read_u64_le(data, 72)?,
                size: read_u64_le(data, 80)?,
            },
            data: BufferSection {
                offset: read_u64_le(data, 88)?,
                size: read_u64_le(data, 96)?,
            },
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
    ///
    /// # Errors
    /// Returns [`ParseError::FileTooSmall`] if `data` is shorter than [`Self::SIZE`].
    pub fn from_bytes(data: &[u8]) -> Result<Self, ParseError> {
        if data.len() < Self::SIZE {
            return Err(ParseError::FileTooSmall {
                expected: Self::SIZE,
                actual: data.len(),
            });
        }

        Ok(Self {
            owner_offset: read_u64_le(data, 0)?,
            name_offset: read_u64_le(data, 8)?,
            type_offset: read_u64_le(data, 16)?,
            data_offset: read_u64_le(data, 24)?,
            bytes_size: read_u64_le(data, 32)?,
        })
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
    ///
    /// # Errors
    /// Returns [`ParseError::FileTooSmall`] if `data` is shorter than [`Self::SIZE`].
    pub fn from_bytes(data: &[u8]) -> Result<Self, ParseError> {
        if data.len() < Self::SIZE {
            return Err(ParseError::FileTooSmall {
                expected: Self::SIZE,
                actual: data.len(),
            });
        }

        Ok(Self {
            owner_offset: read_u64_le(data, 0)?,
            name_offset: read_u64_le(data, 8)?,
            data_offset: read_u64_le(data, 16)?,
            size: read_u64_le(data, 24)?,
        })
    }
}
