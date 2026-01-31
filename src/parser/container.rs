//! Container layer: encryption and compression wrapper.

use super::header::ParseError;
use super::{read_u16_le, read_u32_le, read_u64_le};

/// Container header (packed, 80 bytes total with alignment).
#[derive(Debug)]
pub struct ContainerHeader {
    pub ident: u32, // Should be 0x53504330 ("0CPS" as bytes, "SPC0" as string)
    pub checksum: u32,
    pub num_buffers: u64,
    pub buffers_table_ofs: u64,
    pub seed: u32,
    pub buffers_data_ofs: u64,
}

impl ContainerHeader {
    pub const MAGIC: u32 = 0x53504330; // "0CPS" as stored bytes (reads as "SPC0")
    pub const SIZE: usize = 80; // 4+4+8+8+4+8+10*4 = 80 bytes with reserved

    /// Parse a container header from raw bytes.
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
            ident: read_u32_le(data, 0)?,
            checksum: read_u32_le(data, 4)?,
            num_buffers: read_u64_le(data, 8)?,
            buffers_table_ofs: read_u64_le(data, 16)?,
            seed: read_u32_le(data, 24)?,
            // Note: bytes 28-32 have padding due to alignment
            buffers_data_ofs: read_u64_le(data, 32)?,
            // Reserved: 40-80
        })
    }
}

/// Buffer entry in the table (24 bytes with 8-byte alignment).
/// C++ struct is: u8 encoding, 7-byte padding, u64 offset, u64 size
#[derive(Debug, Clone, Copy)]
pub struct BufferEntry {
    pub encoding: u8,
    pub offset: u64,
    pub size: u64,
}

impl BufferEntry {
    pub const SIZE: usize = 24; // 1 + 7 padding + 8 + 8 = 24 bytes

    /// Parse a buffer entry from raw bytes.
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
            encoding: *data.first().ok_or(ParseError::FileTooSmall {
                expected: 1,
                actual: data.len(),
            })?,
            // 7 bytes padding at 1-7
            offset: read_u64_le(data, 8)?,
            size: read_u64_le(data, 16)?,
        })
    }
}

/// Decrypt the data (XOR-based with avalanche).
pub fn decrypt(data: &mut [u8], encryption_key: u32, seed: u32, block_size: usize) {
    if block_size == 0 || data.len() < 4 {
        return;
    }

    let num_elements = data.len() / 4;
    let key = encryption_key ^ seed;

    // Helper: replicate byte across u32
    let repmat = |value: u32| -> u32 {
        let v = value & 0xFF;
        let v = v | (v << 8);
        let v = v | (v << 16);
        !v
    };

    let mut current_key = key.wrapping_add(repmat(num_elements as u32));

    // Process as u32 words
    let words: &mut [u32] =
        unsafe { std::slice::from_raw_parts_mut(data.as_mut_ptr().cast::<u32>(), num_elements) };

    for j in 0..block_size {
        let mut i = j;
        while i < num_elements {
            let temp = !words[i];
            words[i] ^= current_key;
            current_key = current_key.wrapping_add(temp);
            current_key = current_key.wrapping_add(repmat(i as u32));
            i += block_size;
        }
    }
}

/// Compute checksum (for verification).
#[must_use]
pub fn checksum(data: &[u8]) -> u32 {
    let mut sum: u32 = 0;
    let mut i = 0;

    // Add u32s
    while let Ok(val) = read_u32_le(data, i) {
        sum = sum.wrapping_add(!val);
        i += 4;
    }

    // Add u16s
    while let Ok(val) = read_u16_le(data, i) {
        sum = sum.wrapping_add(u32::from(!val));
        i += 2;
    }

    // Add u8s
    while i < data.len() {
        sum = sum.wrapping_add(u32::from(!data[i]));
        i += 1;
    }

    !sum
}

/// RLE8 decode: pairs of (count, byte).
#[must_use]
pub fn rle8_decode(data: &[u8]) -> Vec<u8> {
    let mut result = Vec::new();
    let mut i = 0;

    while i + 1 < data.len() {
        let count = data[i] as usize;
        let symbol = data[i + 1];
        result.extend(std::iter::repeat_n(symbol, count));
        i += 2;
    }

    result
}

/// RLE0 decode: variable block size RLE.
#[must_use]
pub fn rle0_decode(data: &[u8]) -> Vec<u8> {
    let mut result = Vec::new();
    let mut block_size: usize = 1;
    let mut i = 0;

    while i < data.len() {
        let occurrence = data[i] as usize;
        i += 1;

        // Check if command byte (0 means read new block size)
        if occurrence == 0 {
            if i >= data.len() {
                break;
            }
            block_size = data[i] as usize;
            i += 1;

            if i >= data.len() {
                break;
            }
            let occurrence = data[i] as usize;
            i += 1;

            if i + block_size > data.len() {
                break;
            }
            let block = &data[i..i + block_size];
            for _ in 0..occurrence {
                result.extend_from_slice(block);
            }
            i += block_size;
        } else {
            if i + block_size > data.len() {
                break;
            }
            let block = &data[i..i + block_size];
            for _ in 0..occurrence {
                result.extend_from_slice(block);
            }
            i += block_size;
        }
    }

    result
}

/// Decode based on encoding type.
#[must_use]
pub fn decode(data: &[u8], encoding: u8) -> Vec<u8> {
    match encoding {
        0 => data.to_vec(),     // ENCODING_NONE
        1 => rle8_decode(data), // ENCODING_RLE8
        2 => rle0_decode(data), // ENCODING_RLE0
        _ => data.to_vec(),     // Unknown, return as-is
    }
}

/// Unpack a container: decrypt, decompress, and return `StorageObject` data.
///
/// # Errors
/// Returns a [`ParseError`] if the input does not match the expected container format or if any
/// contained buffer offsets are invalid.
pub fn unpack_container(data: &[u8]) -> Result<Vec<Vec<u8>>, ParseError> {
    const ENCRYPTION_KEY: u32 = 0xfeedbeef;
    const BLOCK_SIZE: usize = 4;

    let header = ContainerHeader::from_bytes(data)?;

    if header.ident != ContainerHeader::MAGIC {
        return Err(ParseError::TypeMismatch {
            expected: format!("SPC0 magic (0x{:08X})", ContainerHeader::MAGIC),
            actual: format!("0x{:08X}", header.ident),
        });
    }

    // Make a mutable copy for decryption
    let mut data = data.to_vec();

    // Zero out checksum for verification
    data[4..8].copy_from_slice(&[0, 0, 0, 0]);

    // Decrypt everything after header
    if data.len() > ContainerHeader::SIZE {
        decrypt(
            &mut data[ContainerHeader::SIZE..],
            ENCRYPTION_KEY,
            header.seed,
            BLOCK_SIZE,
        );
    }

    // Verify checksum
    let computed = checksum(&data);
    if computed != header.checksum {
        return Err(ParseError::TypeMismatch {
            expected: format!("checksum 0x{:08X}", header.checksum),
            actual: format!("0x{computed:08X}"),
        });
    }

    // Parse buffer table
    let table_start = header.buffers_table_ofs as usize;
    let data_start = header.buffers_data_ofs as usize;

    let mut buffers = Vec::new();

    for i in 0..header.num_buffers as usize {
        let entry_start = table_start + i * BufferEntry::SIZE;
        let entry_bytes = data
            .get(entry_start..entry_start + BufferEntry::SIZE)
            .ok_or(ParseError::InvalidOffset {
                offset: entry_start as u64,
                size: data.len(),
            })?;
        let entry = BufferEntry::from_bytes(entry_bytes)?;

        let buf_start = data_start + entry.offset as usize;
        let buf_end = buf_start + entry.size as usize;

        if buf_end > data.len() {
            return Err(ParseError::InvalidOffset {
                offset: buf_end as u64,
                size: data.len(),
            });
        }

        let encoded_data = &data[buf_start..buf_end];
        let decoded_data = decode(encoded_data, entry.encoding);
        buffers.push(decoded_data);
    }

    Ok(buffers)
}
