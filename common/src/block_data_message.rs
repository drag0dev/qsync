use serde::{Serialize, Deserialize};
use anyhow::{Result, Context};

#[derive(Debug, Serialize, Deserialize)]
pub struct BlockDataMessage {
    pub block_idx: u64,
    pub file_path: String,
    pub block_data: Vec<u8>
}

impl BlockDataMessage {
    pub fn new(block_idx: u64, file_path: String, block_data: Vec<u8>) -> Self {
        BlockDataMessage { block_idx, file_path, block_data }
    }

    pub fn serialize(&self) -> Result<Vec<u8>> {
        Ok(bincode::serialize(&self).context("serializing block data")?)
    }

    pub fn deserialize(msg: &[u8]) -> Result<Self> {
        Ok(bincode::deserialize(msg).context("deserializing block data")?)
    }
}
