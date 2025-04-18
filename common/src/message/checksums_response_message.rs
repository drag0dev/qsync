use super::{
    super::file_meta::FileMeta,
    SerializableMessage
};
use serde::{Serialize, Deserialize};
use anyhow::{Result, Context};

#[derive(Debug, Serialize, Deserialize)]
pub struct ChecksumsResponseMessage {
    pub checksums: Vec<FileMeta>
}

impl ChecksumsResponseMessage {
    pub fn new(checksums: Vec<FileMeta>) -> Self {
        ChecksumsResponseMessage { checksums }
    }

    pub fn deseralize(msg: &[u8]) -> Result<Self> {
        Ok(bincode::deserialize(&msg).context("deserializing checksums response")?)
    }
}

impl SerializableMessage for ChecksumsResponseMessage {
    fn serialize(&self) -> Result<Vec<u8>> {
        Ok(bincode::serialize(&self).context("serializing sync request message")?)
    }
}
