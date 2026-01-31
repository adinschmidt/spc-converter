//! `StorageObject` reconstruction from binary format.

use super::header::{PackChild, PackHeader, PackVar, ParseError};
use std::collections::HashMap;

/// A variable stored in the object.
#[derive(Debug, Clone)]
pub struct Variable {
    pub owner: String,
    pub name: String,
    pub type_name: String,
    pub data: Vec<u8>,
}

/// Reconstructed `StorageObject` from binary format.
#[derive(Debug, Clone)]
pub struct StorageObject {
    pub type_name: String,
    pub owner_name: String,
    pub var_name: String,
    pub variables: Vec<Variable>,
    pub children: Vec<Self>,
}

impl StorageObject {
    /// Parse a `StorageObject` from raw bytes.
    ///
    /// # Errors
    /// Returns a [`ParseError`] if the input data is malformed or contains out-of-range offsets.
    pub fn from_bytes(data: &[u8]) -> Result<Self, ParseError> {
        let header = PackHeader::from_bytes(data)?;

        let strings_section = checked_slice(data, header.strings.offset, header.strings.size)?;
        let vars_section = checked_slice(data, header.vars.offset, header.vars.size)?;
        let children_section = checked_slice(data, header.children.offset, header.children.size)?;
        let data_section = checked_slice(data, header.data.offset, header.data.size)?;

        let type_name = read_string(strings_section, header.type_name_offset)?;
        let owner_name = read_string(strings_section, header.owner_offset)?;
        let var_name = read_string(strings_section, header.name_offset)?;

        let variables =
            parse_variables(strings_section, vars_section, data_section, header.num_vars)?;
        let children = parse_children(children_section, data_section, header.num_children)?;

        Ok(Self {
            type_name,
            owner_name,
            var_name,
            variables,
            children,
        })
    }

    /// Find a variable by name.
    #[must_use]
    pub fn find_var(&self, name: &str) -> Option<&Variable> {
        self.variables.iter().find(|v| v.name == name)
    }

    /// Find a child object by variable name.
    #[must_use]
    pub fn find_child(&self, var_name: &str) -> Option<&Self> {
        self.children.iter().find(|c| c.var_name == var_name)
    }

    /// Get all variables as a map by name.
    #[must_use]
    pub fn vars_by_name(&self) -> HashMap<&str, &Variable> {
        self.variables
            .iter()
            .map(|v| (v.name.as_str(), v))
            .collect()
    }
}

fn checked_slice(data: &[u8], offset: u64, size: u64) -> Result<&[u8], ParseError> {
    let end_offset = offset.saturating_add(size);
    let start = usize::try_from(offset).map_err(|_| ParseError::InvalidOffset {
        offset,
        size: data.len(),
    })?;
    let size_usize = usize::try_from(size).map_err(|_| ParseError::InvalidOffset {
        offset: end_offset,
        size: data.len(),
    })?;
    let end = start
        .checked_add(size_usize)
        .ok_or(ParseError::InvalidOffset {
            offset: end_offset,
            size: data.len(),
        })?;

    data.get(start..end).ok_or(ParseError::InvalidOffset {
        offset: end_offset,
        size: data.len(),
    })
}

fn parse_variables(
    strings_section: &[u8],
    vars_section: &[u8],
    data_section: &[u8],
    num_vars: u64,
) -> Result<Vec<Variable>, ParseError> {
    let num_vars_usize = usize::try_from(num_vars).map_err(|_| ParseError::InvalidOffset {
        offset: num_vars,
        size: vars_section.len(),
    })?;
    let expected_vars_size =
        num_vars_usize
            .checked_mul(PackVar::SIZE)
            .ok_or(ParseError::InvalidOffset {
                offset: num_vars,
                size: vars_section.len(),
            })?;
    if vars_section.len() != expected_vars_size {
        return Err(ParseError::VarCountMismatch {
            expected: num_vars,
            actual: vars_section.len() / PackVar::SIZE,
        });
    }

    let mut variables = Vec::with_capacity(num_vars_usize);
    for i in 0..num_vars_usize {
        let start = i * PackVar::SIZE;
        let end = start + PackVar::SIZE;
        let var_bytes = vars_section
            .get(start..end)
            .ok_or(ParseError::FileTooSmall {
                expected: end,
                actual: vars_section.len(),
            })?;
        let pack_var = PackVar::from_bytes(var_bytes)?;

        let owner = read_string(strings_section, pack_var.owner_offset)?;
        let name = read_string(strings_section, pack_var.name_offset)?;
        let type_name = read_string(strings_section, pack_var.type_offset)?;
        let data = checked_slice(data_section, pack_var.data_offset, pack_var.bytes_size)?.to_vec();

        variables.push(Variable {
            owner,
            name,
            type_name,
            data,
        });
    }

    Ok(variables)
}

fn parse_children(
    children_section: &[u8],
    data_section: &[u8],
    num_children: u64,
) -> Result<Vec<StorageObject>, ParseError> {
    let num_children_usize =
        usize::try_from(num_children).map_err(|_| ParseError::InvalidOffset {
            offset: num_children,
            size: children_section.len(),
        })?;
    let expected_children_size =
        num_children_usize
            .checked_mul(PackChild::SIZE)
            .ok_or(ParseError::InvalidOffset {
                offset: num_children,
                size: children_section.len(),
            })?;
    if children_section.len() != expected_children_size {
        return Err(ParseError::ChildCountMismatch {
            expected: num_children,
            actual: children_section.len() / PackChild::SIZE,
        });
    }

    let mut children = Vec::with_capacity(num_children_usize);
    for i in 0..num_children_usize {
        let start = i * PackChild::SIZE;
        let end = start + PackChild::SIZE;
        let child_bytes = children_section
            .get(start..end)
            .ok_or(ParseError::FileTooSmall {
                expected: end,
                actual: children_section.len(),
            })?;
        let pack_child = PackChild::from_bytes(child_bytes)?;

        let child_data = checked_slice(data_section, pack_child.data_offset, pack_child.size)?;
        children.push(StorageObject::from_bytes(child_data)?);
    }

    Ok(children)
}

/// Read a null-terminated string from the strings section.
fn read_string(strings: &[u8], offset: u64) -> Result<String, ParseError> {
    let start = usize::try_from(offset).map_err(|_| ParseError::InvalidOffset {
        offset,
        size: strings.len(),
    })?;
    if start >= strings.len() {
        return Err(ParseError::InvalidOffset {
            offset,
            size: strings.len(),
        });
    }

    let slice = &strings[start..];
    let end = slice
        .iter()
        .position(|&b| b == 0)
        .ok_or(ParseError::UnterminatedString(offset))?;

    String::from_utf8(slice[..end].to_vec()).map_err(|_| ParseError::UnterminatedString(offset))
}
