use super::super::file_meta::FileMeta;
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

    pub fn seralize(&self) -> Result<Vec<u8>> {
        Ok(bincode::serialize(&self).context("serializing checksums response")?)
    }

    pub fn deseralize(msg: &[u8]) -> Result<Self> {
        Ok(bincode::deserialize(&msg).context("deserializing checksums response")?)
    }
}
