use anyhow::{Context, Result};
use super::{MessageHeader, MessageType, SerializableMessage};

pub async fn message_serialize_and_frame<T>(message_type: MessageType, message: &T, compress: bool) -> Result<Vec<u8>>
    where T: SerializableMessage
{
        let msg_ser = message.serialize()?;

        let mut msg_ser = if compress {
            tokio::task::spawn_blocking(move || {
                zstd::encode_all(msg_ser.as_slice(), 0)
            }).await.context("compressing message")??
        } else { msg_ser };

        let header = MessageHeader::new(message_type, msg_ser.len() as u64);
        let mut header_ser = header.serialize()?;

        header_ser.append(&mut msg_ser);
        Ok(header_ser)
}
