use crate::file::DirMeta;
use super::{
    super::file::FileMeta,
    SerializableMessage
};
use serde::{Serialize, Deserialize};
use anyhow::{Result, Context};

#[derive(Debug, Serialize, Deserialize)]
pub struct ChecksumsResponseMessage {
    pub directories: Vec<DirMeta>,
    pub files: Vec<FileMeta>
}

impl ChecksumsResponseMessage {
    pub fn new(files: Vec<FileMeta>, directories: Vec<DirMeta>) -> Self {
        ChecksumsResponseMessage { files, directories }
    }

    pub fn deserialize(msg: &[u8]) -> Result<Self> {
        Ok(bincode::deserialize(&msg).context("deserializing checksums response")?)
    }
}

impl SerializableMessage for ChecksumsResponseMessage {
    fn serialize(&self) -> Result<Vec<u8>> {
        Ok(bincode::serialize(&self).context("serializing sync request message")?)
    }
}
