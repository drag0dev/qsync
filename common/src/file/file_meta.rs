use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct FileMeta {
    pub path: String,
    pub checksums: Vec<String>,

    /// timestamp in ms
    pub modified_timestamp: u128,

    pub size: u64,
}

impl FileMeta {
    pub fn new(path: String, checksums: Vec<String>, modified_timestamp: u128, size: u64) -> Self {
        FileMeta { path, checksums, modified_timestamp, size }
    }
}
