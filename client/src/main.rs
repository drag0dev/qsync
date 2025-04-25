use clap::Parser;
use quinn::Connection;
use std::{
    path::PathBuf,
    sync::Arc,
};
use anyhow::{Result, Context};
use common::{
    file::{generate_temp_entry_point, FileAssembler, FileMeta},
    helpers::unroll_anyhow_result
};
use futures::StreamExt;

mod skip_cert;
mod client;
mod command;
mod helpers;
use client::{send_block_request, send_sync_request};
use command::Command;

// TODO: close connection on early returns

#[tokio::main]
async fn main() -> Result<()> {
    let cmd = Command::parse();

    let local_path = PathBuf::from(&cmd.local_path);
    if !local_path.exists() {
        println!("Error: local path is not valid");
        return Ok(());
    }

    let remote_path = PathBuf::from(&cmd.remote_path);
    if !remote_path.is_absolute() {
        println!("Error: remote path has to be absolute");
        return Ok(());
    }

    rustls::crypto::aws_lc_rs::default_provider().install_default().expect("installing aws_ls_rs");

    let connection = helpers::get_connection()
        .await
        .context("connecting to the server");
    if let Err(e) = connection {
        println!("{}", unroll_anyhow_result(e));
        return Ok(());
    }
    let connection = connection.unwrap();

    println!("Connected to server: {:?}", connection.remote_address());

    let (mut send, mut recv) = connection
        .open_bi()
        .await
        .context("opening bi stream for sync request")?;

    let checksums = send_sync_request(&mut send, &mut recv, &cmd.remote_path).await.context("sending sync request message");
    if let Err(e) = checksums {
        println!("{}", unroll_anyhow_result(e));
        return Ok(());
    }

    let checksums = checksums.unwrap();
    if checksums.is_none() {
        println!("Erorr: remote path does not exist ");
        return Ok(())
    }
    let checksums = checksums.unwrap();

    let temp_entry = generate_temp_entry_point(&cmd.local_path, &cmd.remote_path, &checksums.checksums)
        .context("generating temp entry point");
    if let Err(e) = temp_entry {
        println!("{}", unroll_anyhow_result(e));
        return Ok(());
    }
    let temp_entry = temp_entry.unwrap();

    let entry_point_path = Arc::new(temp_entry);
    let remote_target_path = Arc::new(remote_path);
    let connection = Arc::new(connection);
    let file_syncing_results: Vec<Result<bool>> = futures::stream::iter(checksums.checksums)
        .map(|checksum| { sync_file(checksum, entry_point_path.clone(), remote_target_path.clone(), connection.clone()) })
        .buffered(cmd.concurrent_streams)
        .collect()
        .await;

    let err = file_syncing_results.into_iter().find(|res| res.is_err());
    if let Some(Err(e)) = err {
        println!("Error: {}", unroll_anyhow_result(e));
        return Ok(());
    }

    connection.close(0u32.into(), b"Done");

    Ok(())
}

async fn sync_file(
    file_meta: FileMeta, entry_point_path: Arc<PathBuf>,
    remote_target_path: Arc<PathBuf>, connection: Arc<Connection>
) -> Result<bool> {
    let local_file = &file_meta.path;

    // both unwraps are safe, because both have been done previously in generate_temp_entry_point
    let local_file = local_file.strip_prefix(remote_target_path.to_str().unwrap()).unwrap();
    let local_file = if local_file.starts_with("/") { local_file.strip_prefix("/").unwrap() } else { local_file };
    let mut local_file_path = entry_point_path.as_ref().clone();

    // when the target is just a file stripping prefix would leave us with an empty path and
    // push an empty string causes it to add a trailing / making it a directory
    if local_file.len() > 0 { local_file_path.push(local_file); }

    let mut file_assembler = FileAssembler::new(local_file_path.to_str().unwrap()).await?;

    for (block_idx, _block_checksum) in file_meta.checksums.iter().enumerate() {
        let (mut send, mut recv) = connection
            .open_bi()
            .await
            .context("opening bi stream")?;

        let block = send_block_request(&mut send, &mut recv, &file_meta.path, block_idx as u64)
            .await
            .context("getting block")?;

        if block.is_none() { return Ok(false); }
        let block = block.unwrap();

        file_assembler
            .write_next_block(&block.block_data)
            .await?;
    }

    Ok(true)
}
