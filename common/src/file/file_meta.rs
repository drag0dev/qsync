use serde::{Deserialize, Serialize};
use anyhow::{Result, Context};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct FileMeta {
    pub path: String,
    pub checksums: Vec<String>
}

impl FileMeta {
    pub fn new(path: String, checksums: Vec<String>) -> Self {
        FileMeta { path, checksums }
    }

    pub fn serialize(&self) -> Result<Vec<u8>> {
        Ok(bincode::serialize(&self).context("serializing file meta")?)
    }

    pub fn deserialize(msg: &[u8]) -> Result<Self> {
        Ok(bincode::deserialize(&msg).context("deserializing file meta")?)
    }
}
