use std::path::Path;
use anyhow::{Result, Context};
use quinn::{RecvStream, SendStream};
use common::{
    file::read_block,
    message::{
        message_serialize_and_frame, BlockDataMessage, BlockRequestMessage,
        ChecksumsResponseMessage, ErrorMessage, MessageHeader, MessageType, SyncRequestMessage
}};
use crate::error_message;

pub async fn handle_sync_request(mut tx: SendStream, mut rx: RecvStream, header: &MessageHeader) -> Result<()> {
    let msg_ser = rx.read_to_end(header.msg_len as usize)
        .await
        .context("reading handle sync request");

    if let Err(e) = msg_ser {
        error_message!(tx, "Internal server error");
        return Err(e);
    }
    let msg_ser = msg_ser.unwrap();

    let sync_requst_msg = SyncRequestMessage::deserialize(&msg_ser);
    if let Err(_) = sync_requst_msg {
        error_message!(tx, "Malformed sync request message");
        return Ok(());
    }
    let sync_requst_msg = sync_requst_msg.unwrap();

    let path = Path::new(&sync_requst_msg.path);
    if !path.exists() {
        error_message!(tx, "Path does not exist");
        return Ok(());
    }

    let checksums = tokio::task::spawn_blocking(move || {
        common::file::get_checksums(&sync_requst_msg.path)
    }).await
    .context("running checksums iterator");

    let checksums = match checksums {
        Err(e) => {
            error_message!(tx, "Internal server error");
            return Err(e);
        },
        Ok(inner) =>
            match inner {
                Err(e) => {
                    error_message!(tx, "Internal server error");
                    return Err(e);
                },
                Ok(checksums) => checksums
            }
    };

    let msg = ChecksumsResponseMessage::new(checksums);
    let msg_ser = message_serialize_and_frame(MessageType::ChecksumsResponse, &msg);

    if let Err(e) = msg_ser {
        error_message!(tx, "Internal server error");
        return Err(e);
    }
    let msg_ser = msg_ser.unwrap();

    // no need to send error message because sending is already failing
    tx.write_all(&msg_ser).await.context("writing checksum response msg")?;

    let _ = tx.finish();

    Ok(())
}

pub async fn handle_block_request(mut tx: SendStream, mut rx: RecvStream, header: &MessageHeader) -> Result<()> {
    let msg_ser = rx.read_to_end(header.msg_len as usize)
        .await
        .context("reading handle sync request");

    if let Err(e) = msg_ser {
        error_message!(tx, "Internal server error");
        return Err(e);
    }
    let msg_ser = msg_ser.unwrap();

    let block_request_msg = BlockRequestMessage::deserialize(&msg_ser);
    if let Err(e) = block_request_msg {
        error_message!(tx, "Malformed block request message");
        return Err(e);
    }
    let block_request_msg = block_request_msg.unwrap();

    let path = Path::new(&block_request_msg.file_path);
    if !path.exists() {
        error_message!(tx, "Path does not exist");
        return Ok(());
    }

    let block_data = read_block(&block_request_msg.file_path, block_request_msg.block_idx)
        .await
        .context("reading block data");

    if let Err(e) = block_data {
        error_message!(tx, "Internal server error");
        return Err(e);
    }
    let block_data = block_data.unwrap();

    let msg = BlockDataMessage::new(block_request_msg.block_idx, block_request_msg.file_path.into(), block_data);

    let msg_ser = message_serialize_and_frame(MessageType::BlockData, &msg);
    if let Err(e) = msg_ser {
        error_message!(tx, "Internal server error");
        return Err(e);
    }
    let msg_ser = msg_ser.unwrap();

    tx.write_all(&msg_ser).await.context("writing block data response msg")?;

    let _ = tx.finish();

    Ok(())
}
