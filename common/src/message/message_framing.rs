use anyhow::Result;
use super::{MessageHeader, MessageType, SerializableMessage};

pub fn message_serialize_and_frame<T>(message_type: MessageType, message: &T) -> Result<Vec<u8>>
    where T: SerializableMessage
{
        let mut msg_ser = message.serialize()?;
        let header = MessageHeader::new(message_type, msg_ser.len() as u64);
        let mut header_ser = header.serialize()?;
        header_ser.append(&mut msg_ser);
        Ok(header_ser)
}
