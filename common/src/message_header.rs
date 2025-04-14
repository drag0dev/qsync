use super::message_type::MessageType;
use deku::{DekuRead, DekuWrite};

#[derive(Debug, DekuRead, DekuWrite)]
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
}
