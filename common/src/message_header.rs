use super::message_type::MessageType;
use serde::{Deserialize, Serialize};
use anyhow::{Result, Context};

pub static HEADER_LEN: usize = 8 + 4;

#[derive(Debug, Serialize, Deserialize)]
pub struct MessageHeader {
    pub msg_len: u64,
    pub msg_type: MessageType,
}

impl MessageHeader {
    pub fn new(msg_type: MessageType, msg_len: u64) -> Self {
        MessageHeader {
            msg_type,
            msg_len,
        }
    }

    pub fn serialize(&self) -> Result<Vec<u8>> {
        Ok(bincode::serialize(&self).context("serializing message header")?)
    }

    pub fn deserialize(header: &[u8]) -> Result<Self> {
        Ok(bincode::deserialize(header).context("deserializing message header")?)
    }
}
