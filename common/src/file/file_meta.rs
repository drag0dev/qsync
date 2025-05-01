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
}

impl FileMeta {
    pub fn new(path: String, checksums: Vec<String>, modified_timestamp: u128, size: u64, permissions: Option<u32>) -> Self {
        FileMeta { path, checksums, modified_timestamp, size, permissions }
    }
}
