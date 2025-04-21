use std::path::Path;
use anyhow::{Result, Context};
use quinn::{RecvStream, SendStream};
use common::{
    file::read_block,
    message::{
        message_serialize_and_frame, BlockDataMessage, BlockRequestMessage,
        ChecksumsResponseMessage, ErrorMessage, MessageHeader, MessageType, SyncRequestMessage
}};

pub async fn handle_sync_request(mut tx: SendStream, mut rx: RecvStream, header: &MessageHeader) -> Result<()> {
    let msg_ser = rx.read_to_end(header.msg_len as usize)
        .await
        .context("reading handle sync request")?;

    let sync_requst_msg = SyncRequestMessage::deserialize(&msg_ser)?;

    let path = Path::new(&sync_requst_msg.path);
    if !path.exists() {
        let msg = ErrorMessage::new("Path does not exist".into());
        let msg_ser = message_serialize_and_frame(MessageType::Error, &msg)?;
        tx.write_all(&msg_ser).await.context("writing error msg")?;
        return Ok(());
    }

    let checksums = tokio::task::spawn_blocking(move || {
        common::file::get_checksums(&sync_requst_msg.path)
    }).await
    .context("running checksums iterator");

    let checksums = match checksums {
        Err(e) => {
            let msg = ErrorMessage::new("Internal server error".into());
            let msg_ser = message_serialize_and_frame(MessageType::Error, &msg)?;
            tx.write_all(&msg_ser).await.context("writing error msg")?;
            return Err(e);
        },
        Ok(inner) =>
            match inner {
                Err(e) => {
                    let msg = ErrorMessage::new("Internal server error".into());
                    let msg_ser = message_serialize_and_frame(MessageType::Error, &msg)?;
                    tx.write_all(&msg_ser).await.context("writing error msg")?;
                    return Err(e);
                },
                Ok(checksums) => checksums
            }
    };

    let msg = ChecksumsResponseMessage::new(checksums);
    let msg_ser = message_serialize_and_frame(MessageType::ChecksumsResponse, &msg)?;
    tx.write_all(&msg_ser).await.context("writing checksum response msg")?;

    let _ = tx.finish();

    Ok(())
}

// TODO: write intenral server error on internal errors

pub async fn handle_block_request(mut tx: SendStream, mut rx: RecvStream, header: &MessageHeader) -> Result<()> {
    let msg_ser = rx.read_to_end(header.msg_len as usize)
        .await
        .context("reading handle sync request")?;

    let block_request_msg = BlockRequestMessage::deserialize(&msg_ser)?;

    let path = Path::new(&block_request_msg.file_path);
    if !path.exists() {
        let msg = ErrorMessage::new("Path does not exist".into());
        let msg_ser = message_serialize_and_frame(MessageType::Error, &msg)?;
        tx.write_all(&msg_ser).await.context("writing error msg")?;
        return Ok(());
    }

    let block_data = read_block(&block_request_msg.file_path, block_request_msg.block_idx)
        .await
        .context("reading block data")?;

    let msg = BlockDataMessage::new(block_request_msg.block_idx, block_request_msg.file_path.into(), block_data);
    let msg_ser = message_serialize_and_frame(MessageType::BlockData, &msg)?;
    tx.write_all(&msg_ser).await.context("writing block data response msg")?;

    let _ = tx.finish();

    Ok(())
}
