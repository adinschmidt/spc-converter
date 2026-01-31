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

        // Extract strings section
        let strings_start =
            usize::try_from(header.strings.offset).map_err(|_| ParseError::InvalidOffset {
                offset: header.strings.offset,
                size: data.len(),
            })?;
        let strings_size =
            usize::try_from(header.strings.size).map_err(|_| ParseError::InvalidOffset {
                offset: header.strings.offset + header.strings.size,
                size: data.len(),
            })?;
        let strings_end =
            strings_start
                .checked_add(strings_size)
                .ok_or(ParseError::InvalidOffset {
                    offset: header.strings.offset + header.strings.size,
                    size: data.len(),
                })?;
        let strings_section =
            data.get(strings_start..strings_end)
                .ok_or(ParseError::InvalidOffset {
                    offset: header.strings.offset + header.strings.size,
                    size: data.len(),
                })?;

        // Extract data section
        let data_start =
            usize::try_from(header.data.offset).map_err(|_| ParseError::InvalidOffset {
                offset: header.data.offset,
                size: data.len(),
            })?;
        let data_size =
            usize::try_from(header.data.size).map_err(|_| ParseError::InvalidOffset {
                offset: header.data.offset + header.data.size,
                size: data.len(),
            })?;
        let data_end = data_start
            .checked_add(data_size)
            .ok_or(ParseError::InvalidOffset {
                offset: header.data.offset + header.data.size,
                size: data.len(),
            })?;
        let data_section = data
            .get(data_start..data_end)
            .ok_or(ParseError::InvalidOffset {
                offset: header.data.offset + header.data.size,
                size: data.len(),
            })?;

        // Read type name, owner, var name
        let type_name = read_string(strings_section, header.type_name_offset)?;
        let owner_name = read_string(strings_section, header.owner_offset)?;
        let var_name = read_string(strings_section, header.name_offset)?;

        // Parse variables
        let vars_start =
            usize::try_from(header.vars.offset).map_err(|_| ParseError::InvalidOffset {
                offset: header.vars.offset,
                size: data.len(),
            })?;
        let vars_size =
            usize::try_from(header.vars.size).map_err(|_| ParseError::InvalidOffset {
                offset: header.vars.offset + header.vars.size,
                size: data.len(),
            })?;
        let vars_end = vars_start
            .checked_add(vars_size)
            .ok_or(ParseError::InvalidOffset {
                offset: header.vars.offset + header.vars.size,
                size: data.len(),
            })?;
        let vars_section = data
            .get(vars_start..vars_end)
            .ok_or(ParseError::InvalidOffset {
                offset: header.vars.offset + header.vars.size,
                size: data.len(),
            })?;

        let num_vars = usize::try_from(header.num_vars).map_err(|_| ParseError::InvalidOffset {
            offset: header.num_vars,
            size: data.len(),
        })?;
        let expected_vars_size =
            num_vars
                .checked_mul(PackVar::SIZE)
                .ok_or(ParseError::InvalidOffset {
                    offset: header.num_vars,
                    size: data.len(),
                })?;
        if vars_size != expected_vars_size {
            return Err(ParseError::VarCountMismatch {
                expected: header.num_vars,
                actual: vars_size / PackVar::SIZE,
            });
        }

        let mut variables = Vec::with_capacity(num_vars);
        for i in 0..num_vars {
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

            let var_data_start =
                usize::try_from(pack_var.data_offset).map_err(|_| ParseError::InvalidOffset {
                    offset: pack_var.data_offset,
                    size: data_section.len(),
                })?;
            let var_data_size =
                usize::try_from(pack_var.bytes_size).map_err(|_| ParseError::InvalidOffset {
                    offset: pack_var.data_offset + pack_var.bytes_size,
                    size: data_section.len(),
                })?;
            let var_data_end =
                var_data_start
                    .checked_add(var_data_size)
                    .ok_or(ParseError::InvalidOffset {
                        offset: pack_var.data_offset + pack_var.bytes_size,
                        size: data_section.len(),
                    })?;
            let var_data = data_section
                .get(var_data_start..var_data_end)
                .ok_or(ParseError::InvalidOffset {
                    offset: pack_var.data_offset + pack_var.bytes_size,
                    size: data_section.len(),
                })?
                .to_vec();

            variables.push(Variable {
                owner,
                name,
                type_name,
                data: var_data,
            });
        }

        // Parse children
        let children_start =
            usize::try_from(header.children.offset).map_err(|_| ParseError::InvalidOffset {
                offset: header.children.offset,
                size: data.len(),
            })?;
        let children_size =
            usize::try_from(header.children.size).map_err(|_| ParseError::InvalidOffset {
                offset: header.children.offset + header.children.size,
                size: data.len(),
            })?;
        let children_end =
            children_start
                .checked_add(children_size)
                .ok_or(ParseError::InvalidOffset {
                    offset: header.children.offset + header.children.size,
                    size: data.len(),
                })?;
        let children_section =
            data.get(children_start..children_end)
                .ok_or(ParseError::InvalidOffset {
                    offset: header.children.offset + header.children.size,
                    size: data.len(),
                })?;

        let num_children =
            usize::try_from(header.num_children).map_err(|_| ParseError::InvalidOffset {
                offset: header.num_children,
                size: data.len(),
            })?;
        let expected_children_size =
            num_children
                .checked_mul(PackChild::SIZE)
                .ok_or(ParseError::InvalidOffset {
                    offset: header.num_children,
                    size: data.len(),
                })?;
        if children_size != expected_children_size {
            return Err(ParseError::ChildCountMismatch {
                expected: header.num_children,
                actual: children_size / PackChild::SIZE,
            });
        }

        let mut children = Vec::with_capacity(num_children);
        for i in 0..num_children {
            let start = i * PackChild::SIZE;
            let end = start + PackChild::SIZE;
            let child_bytes = children_section
                .get(start..end)
                .ok_or(ParseError::FileTooSmall {
                    expected: end,
                    actual: children_section.len(),
                })?;
            let pack_child = PackChild::from_bytes(child_bytes)?;

            let child_data_start =
                usize::try_from(pack_child.data_offset).map_err(|_| ParseError::InvalidOffset {
                    offset: pack_child.data_offset,
                    size: data_section.len(),
                })?;
            let child_data_size =
                usize::try_from(pack_child.size).map_err(|_| ParseError::InvalidOffset {
                    offset: pack_child.data_offset + pack_child.size,
                    size: data_section.len(),
                })?;
            let child_data_end =
                child_data_start
                    .checked_add(child_data_size)
                    .ok_or(ParseError::InvalidOffset {
                        offset: pack_child.data_offset + pack_child.size,
                        size: data_section.len(),
                    })?;
            let child_data = data_section.get(child_data_start..child_data_end).ok_or(
                ParseError::InvalidOffset {
                    offset: pack_child.data_offset + pack_child.size,
                    size: data_section.len(),
                },
            )?;

            // Recursively parse child
            let child_obj = Self::from_bytes(child_data)?;
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
