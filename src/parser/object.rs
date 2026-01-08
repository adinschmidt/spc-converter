//! StorageObject reconstruction from binary format.

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

/// Reconstructed StorageObject from binary format.
#[derive(Debug, Clone)]
pub struct StorageObject {
    pub type_name: String,
    pub owner_name: String,
    pub var_name: String,
    pub variables: Vec<Variable>,
    pub children: Vec<StorageObject>,
}

impl StorageObject {
    /// Parse a StorageObject from raw bytes.
    pub fn from_bytes(data: &[u8]) -> Result<Self, ParseError> {
        let header = PackHeader::from_bytes(data)?;

        // Extract strings section
        let strings_start = header.strings.offset as usize;
        let strings_end = strings_start + header.strings.size as usize;
        if strings_end > data.len() {
            return Err(ParseError::InvalidOffset {
                offset: header.strings.offset + header.strings.size,
                size: data.len(),
            });
        }
        let strings_section = &data[strings_start..strings_end];

        // Extract data section
        let data_start = header.data.offset as usize;
        let data_end = data_start + header.data.size as usize;
        if data_end > data.len() {
            return Err(ParseError::InvalidOffset {
                offset: header.data.offset + header.data.size,
                size: data.len(),
            });
        }
        let data_section = &data[data_start..data_end];

        // Read type name, owner, var name
        let type_name = read_string(strings_section, header.type_name_offset)?;
        let owner_name = read_string(strings_section, header.owner_offset)?;
        let var_name = read_string(strings_section, header.name_offset)?;

        // Parse variables
        let vars_start = header.vars.offset as usize;
        let vars_end = vars_start + header.vars.size as usize;
        if vars_end > data.len() {
            return Err(ParseError::InvalidOffset {
                offset: header.vars.offset + header.vars.size,
                size: data.len(),
            });
        }
        let vars_section = &data[vars_start..vars_end];

        let expected_vars_size = header.num_vars as usize * PackVar::SIZE;
        if header.vars.size as usize != expected_vars_size {
            return Err(ParseError::VarCountMismatch {
                expected: header.num_vars,
                actual: header.vars.size as usize / PackVar::SIZE,
            });
        }

        let mut variables = Vec::with_capacity(header.num_vars as usize);
        for i in 0..header.num_vars as usize {
            let var_bytes = &vars_section[i * PackVar::SIZE..(i + 1) * PackVar::SIZE];
            let pack_var = PackVar::from_bytes(var_bytes);

            let owner = read_string(strings_section, pack_var.owner_offset)?;
            let name = read_string(strings_section, pack_var.name_offset)?;
            let type_name = read_string(strings_section, pack_var.type_offset)?;

            let var_data_start = pack_var.data_offset as usize;
            let var_data_end = var_data_start + pack_var.bytes_size as usize;
            if var_data_end > data_section.len() {
                return Err(ParseError::InvalidOffset {
                    offset: pack_var.data_offset + pack_var.bytes_size,
                    size: data_section.len(),
                });
            }
            let var_data = data_section[var_data_start..var_data_end].to_vec();

            variables.push(Variable {
                owner,
                name,
                type_name,
                data: var_data,
            });
        }

        // Parse children
        let children_start = header.children.offset as usize;
        let children_end = children_start + header.children.size as usize;
        if children_end > data.len() {
            return Err(ParseError::InvalidOffset {
                offset: header.children.offset + header.children.size,
                size: data.len(),
            });
        }
        let children_section = &data[children_start..children_end];

        let expected_children_size = header.num_children as usize * PackChild::SIZE;
        if header.children.size as usize != expected_children_size {
            return Err(ParseError::ChildCountMismatch {
                expected: header.num_children,
                actual: header.children.size as usize / PackChild::SIZE,
            });
        }

        let mut children = Vec::with_capacity(header.num_children as usize);
        for i in 0..header.num_children as usize {
            let child_bytes = &children_section[i * PackChild::SIZE..(i + 1) * PackChild::SIZE];
            let pack_child = PackChild::from_bytes(child_bytes);

            let child_data_start = pack_child.data_offset as usize;
            let child_data_end = child_data_start + pack_child.size as usize;
            if child_data_end > data_section.len() {
                return Err(ParseError::InvalidOffset {
                    offset: pack_child.data_offset + pack_child.size,
                    size: data_section.len(),
                });
            }
            let child_data = &data_section[child_data_start..child_data_end];

            // Recursively parse child
            let child_obj = StorageObject::from_bytes(child_data)?;
            children.push(child_obj);
        }

        Ok(Self {
            type_name,
            owner_name,
            var_name,
            variables,
            children,
        })
    }

    /// Find a variable by name.
    pub fn find_var(&self, name: &str) -> Option<&Variable> {
        self.variables.iter().find(|v| v.name == name)
    }

    /// Find a child object by variable name.
    pub fn find_child(&self, var_name: &str) -> Option<&StorageObject> {
        self.children.iter().find(|c| c.var_name == var_name)
    }

    /// Get all variables as a map by name.
    pub fn vars_by_name(&self) -> HashMap<&str, &Variable> {
        self.variables.iter().map(|v| (v.name.as_str(), v)).collect()
    }
}

/// Read a null-terminated string from the strings section.
fn read_string(strings: &[u8], offset: u64) -> Result<String, ParseError> {
    let start = offset as usize;
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

    String::from_utf8(slice[..end].to_vec())
        .map_err(|_| ParseError::UnterminatedString(offset))
}
