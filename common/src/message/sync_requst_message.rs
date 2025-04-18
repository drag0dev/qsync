use serde::{Serialize, Deserialize};
use anyhow::{Result, Context};
use super::SerializableMessage;

#[derive(Debug, Serialize, Deserialize)]
pub struct SyncRequestMessage {
    pub path: String
}

impl SyncRequestMessage {
    pub fn new(path: String) -> Self {
        Self { path }
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
