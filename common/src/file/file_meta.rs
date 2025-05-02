use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct FileMeta {
    pub path: String,
    pub checksums: Vec<String>,

    /// timestamp in ms
    pub modified_timestamp: u128,

    pub size: u64,

    /// file permissions
    pub permissions: Option<u32>,

    pub is_symlink: bool,
    pub symlink_target: Option<String>,
}

impl FileMeta {
    pub fn new_file(path: String, checksums: Vec<String>, modified_timestamp: u128, size: u64, permissions: Option<u32>) -> Self {
        FileMeta { path, checksums, modified_timestamp, size, permissions, is_symlink: false, symlink_target: None}
    }

    pub fn new_symlink(path: String, symlink_target: String) -> Self {
        FileMeta {
            path,
            checksums: vec![],
            modified_timestamp: 0,
            size: 0,
            permissions: None,
            is_symlink: true,
            symlink_target: Some(symlink_target)
        }
    }
}
