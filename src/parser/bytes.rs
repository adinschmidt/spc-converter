//! Helpers for reading little-endian numbers from byte slices.

use super::header::ParseError;

fn get_slice(data: &[u8], offset: usize, len: usize) -> Result<&[u8], ParseError> {
    let end = offset.checked_add(len).ok_or(ParseError::InvalidOffset {
        offset: offset as u64,
        size: data.len(),
    })?;

    data.get(offset..end).ok_or(ParseError::FileTooSmall {
        expected: end,
        actual: data.len(),
    })
}

pub fn read_u16_le(data: &[u8], offset: usize) -> Result<u16, ParseError> {
    let bytes: [u8; 2] = get_slice(data, offset, 2)?.try_into().map_err(|_| {
        let end = offset + 2;
        ParseError::FileTooSmall {
            expected: end,
            actual: data.len(),
        }
    })?;
    Ok(u16::from_le_bytes(bytes))
}

pub fn read_u32_le(data: &[u8], offset: usize) -> Result<u32, ParseError> {
    let bytes: [u8; 4] = get_slice(data, offset, 4)?.try_into().map_err(|_| {
        let end = offset + 4;
        ParseError::FileTooSmall {
            expected: end,
            actual: data.len(),
        }
    })?;
    Ok(u32::from_le_bytes(bytes))
}

pub fn read_u64_le(data: &[u8], offset: usize) -> Result<u64, ParseError> {
    let bytes: [u8; 8] = get_slice(data, offset, 8)?.try_into().map_err(|_| {
        let end = offset + 8;
        ParseError::FileTooSmall {
            expected: end,
            actual: data.len(),
        }
    })?;
    Ok(u64::from_le_bytes(bytes))
}
