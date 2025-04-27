use serde::{Deserialize, Serialize};
use anyhow::{Result, Context};

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

    pub fn serialize(&self) -> Result<Vec<u8>> {
        Ok(bincode::serialize(&self).context("serializing file meta")?)
    }

    pub fn deserialize(msg: &[u8]) -> Result<Self> {
        Ok(bincode::deserialize(&msg).context("deserializing file meta")?)
    }
}
