use serde::{Serialize, Deserialize};
use anyhow::{Result, Context};
use super::SerializableMessage;

#[derive(Debug, Serialize, Deserialize)]
pub struct BlockRequestMessage {
    pub block_idx: u64,
    pub file_path: String,
}

impl BlockRequestMessage {
    pub fn new(block_idx: u64, file_path: String) -> Self {
        BlockRequestMessage { block_idx, file_path }
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
