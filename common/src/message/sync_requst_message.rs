use serde::{Serialize, Deserialize};
use anyhow::{Result, Context};
use super::SerializableMessage;

#[derive(Debug, Serialize, Deserialize)]
pub enum SyncTargetType {
    File,
    Directory,
    Symlink
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SyncRequestMessage {
    pub path: String,
    pub file_type: SyncTargetType,
    pub block_size: usize,
}

impl SyncRequestMessage {
    pub fn new(path: String, file_type: SyncTargetType, block_size: usize) -> Self {
        Self { path, file_type, block_size }
    }

    pub fn deserialize(msg: &[u8]) -> Result<Self> {
        Ok(bincode::deserialize(&msg).context("deserializing sync request message")?)
    }
}

impl SerializableMessage for SyncRequestMessage {
    fn serialize(&self) -> Result<Vec<u8>> {
        Ok(bincode::serialize(&self).context("serializing sync request message")?)
    }
}
