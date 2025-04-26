use common::message::{
    message_serialize_and_frame, BlockDataMessage,
    BlockRequestMessage, ChecksumsResponseMessage, ErrorMessage,
    MessageHeader, MessageType, SyncRequestMessage, HEADER_LEN
};
use anyhow::{anyhow, Context, Result};
use quinn::{RecvStream, SendStream};
use tokio::io::AsyncWriteExt;

pub async fn send_sync_request(send: &mut SendStream, recv: &mut RecvStream, path: &str, is_dir: bool) -> Result<Option<ChecksumsResponseMessage>> {
    let msg = SyncRequestMessage::new(path.into(), is_dir);
    let msg_ser = message_serialize_and_frame(MessageType::SyncRequest, &msg)?;

    send.write_all(&msg_ser)
        .await
        .context("writing sync request message")?;

    send.flush().await.context("flushing sync request message")?;

    let mut header_buff = [0u8; HEADER_LEN];
    recv.read_exact(&mut header_buff)
        .await
        .context("reading header")?;

    let header = MessageHeader::deserialize(&header_buff)?;

    let msg_ser = recv.read_to_end(header.msg_len as usize)
        .await
        .context("reading checksum message")?;

    match header.msg_type {
        MessageType::Error => {
            let msg = ErrorMessage::deserialize(&msg_ser)?;
            println!("Error sending sync request: {}", msg.message);
            Ok(None)
        }
        MessageType::ChecksumsResponse => {
            let msg = ChecksumsResponseMessage::deserialize(&msg_ser)?;
            Ok(Some(msg))
        }
        _ => Err(anyhow!("Unexpected response from server, got: {}", header.msg_type)),
    }
}

pub async fn send_block_request(send: &mut SendStream, recv: &mut RecvStream, path: &str, block_idx: u64) -> Result<Option<BlockDataMessage>> {
    let msg = BlockRequestMessage::new(block_idx, path.into());
    let msg_ser = message_serialize_and_frame(MessageType::BlockRequest, &msg)?;

    send.write_all(&msg_ser)
        .await
        .context("writing block request message")?;

    send.flush().await.context("flushing block request message")?;

    let mut header_buff = [0u8; HEADER_LEN];
    recv.read_exact(&mut header_buff)
        .await
        .context("reading header")?;

    let header = MessageHeader::deserialize(&header_buff)?;

    let msg_ser = recv.read_to_end(header.msg_len as usize)
        .await
        .context("reading checksum message")?;

    match header.msg_type {
        MessageType::Error => {
            let msg = ErrorMessage::deserialize(&msg_ser)?;
            println!("Error sending block request: {}", msg.message);
            Ok(None)
        }
        MessageType::BlockData => {
            let msg = BlockDataMessage::deserialize(&msg_ser)?;
            Ok(Some(msg))
        }
        _ => Err(anyhow!("Unexpected response from server, got: {}", header.msg_type)),
    }
}
