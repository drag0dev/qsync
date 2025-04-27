use std::path::Path;
use anyhow::{Result, Context};
use quinn::{RecvStream, SendStream};
use common::{
    file::read_block,
    message::{
        message_serialize_and_frame, BlockDataMessage, BlockRequestMessage,
        ChecksumsResponseMessage, ErrorMessage, MessageHeader, MessageType, SyncRequestMessage
}};
use tokio::io::AsyncWriteExt;
use crate::error_message;

pub async fn handle_sync_request(mut tx: SendStream, mut rx: RecvStream, header: &MessageHeader) -> Result<()> {
    let mut msg_ser = vec![0u8; header.msg_len as usize];
    let reading_res = rx.read_exact(&mut msg_ser)
        .await
        .context("reading handle sync request");

    if let Err(e) = reading_res {
        error_message!(tx, "Internal server error");
        return Err(e);
    }

    let sync_requst_msg = SyncRequestMessage::deserialize(&msg_ser);
    if let Err(_) = sync_requst_msg {
        error_message!(tx, "Malformed sync request message");
        return Ok(());
    }
    let sync_requst_msg = sync_requst_msg.unwrap();

    // check if client is requesting a file-file or dir-dir sync
    let res = tokio::fs::metadata(&sync_requst_msg.path)
        .await
        .context("getting path metadata")?;

    if res.is_dir() != sync_requst_msg.is_dir {
        error_message!(tx, "File and a directory cannot be synced");
        return Ok(());
    }

    let path = Path::new(&sync_requst_msg.path);
    if !path.exists() {
        error_message!(tx, "Path does not exist");
        return Ok(());
    }

    let checksums = tokio::task::spawn_blocking(move || {
        common::file::get_checksums(&sync_requst_msg.path, sync_requst_msg.block_size)
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
    let msg_ser = message_serialize_and_frame(MessageType::ChecksumsResponse, &msg, true).await;

    if let Err(e) = msg_ser {
        error_message!(tx, "Internal server error");
        return Err(e);
    }
    let msg_ser = msg_ser.unwrap();

    // no need to send error message because sending is already failing
    tx.write_all(&msg_ser).await.context("writing checksum response msg")?;
    tx.flush().await.context("flushing checksums response")?;

    let _ = tx.finish();

    Ok(())
}

pub async fn handle_block_request(mut tx: SendStream, mut rx: RecvStream, header: &MessageHeader) -> Result<()> {
    let mut msg_ser = vec![0u8; header.msg_len as usize];
    let reading_res = rx.read_exact(&mut msg_ser)
        .await
        .context("reading handle sync request");

    if let Err(e) = reading_res {
        error_message!(tx, "Internal server error");
        return Err(e);
    }

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

    let block_data = read_block(&block_request_msg.file_path, block_request_msg.block_idx, block_request_msg.block_size)
        .await
        .context("reading block data");

    if let Err(e) = block_data {
        error_message!(tx, "Internal server error");
        return Err(e);
    }
    let block_data = block_data.unwrap();

    let msg = BlockDataMessage::new(block_request_msg.block_idx, block_request_msg.file_path.into(), block_data);

    let msg_ser = message_serialize_and_frame(MessageType::BlockData, &msg, true).await;
    if let Err(e) = msg_ser {
        error_message!(tx, "Internal server error");
        return Err(e);
    }
    let msg_ser = msg_ser.unwrap();

    tx.write_all(&msg_ser).await.context("writing block data response msg")?;
    tx.flush().await.context("flushing checksums response")?;

    let _ = tx.finish();

    Ok(())
}
