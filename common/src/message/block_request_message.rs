use serde::{Serialize, Deserialize};
use anyhow::{Result, Context};
use super::SerializableMessage;

#[derive(Debug, Serialize, Deserialize)]
pub struct BlockRequestMessage {
    pub block_idx: u64,
    pub file_path: String,
    pub block_size: usize,
}

impl BlockRequestMessage {
    pub fn new(block_idx: u64, file_path: String, block_size: usize) -> Self {
        BlockRequestMessage { block_idx, file_path, block_size }
    }

    pub fn deserialize(msg: &[u8]) -> Result<Self> {
        Ok(bincode::deserialize(msg).context("deserializing block request")?)
    }
}

impl SerializableMessage for BlockRequestMessage {
    fn serialize(&self) -> Result<Vec<u8>> {
        Ok(bincode::serialize(&self).context("serializing sync request message")?)
    }
}
