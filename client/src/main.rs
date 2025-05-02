use clap::Parser;
use helpers::{clean_up_temp, naive_check};
use quinn::Connection;
use tokio::{
    fs::{remove_file, symlink},
    signal::{self, unix::{signal, SignalKind}},
    sync::Mutex
};
use std::{
    path::{Path, PathBuf},
    sync::{atomic::{AtomicBool, Ordering}, Arc}
};
use anyhow::{Result, Context};
use common::{
    file::{
        generate_temp_entry_point, update_timestamps_on_dirs, AsyncFileChecksumIter,
        FileAssembler, FileMeta
    },
    helpers::unroll_anyhow_result, message::SyncTargetType
};
use futures::StreamExt;

mod skip_cert;
mod client;
mod command;
mod helpers;
use client::{send_block_request, send_sync_request};
use command::Command;

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

    let connection = helpers::get_connection(&cmd.server_address, cmd.port)
        .await
        .context("connecting to the server");
    if let Err(e) = connection {
        println!("{}", unroll_anyhow_result(e));
        return Ok(());
    }
    let connection = connection.unwrap();
    let connection = Arc::new(connection);

    println!("Connected to server: {:?}", connection.remote_address());

    // temp entry point
    let temp_entry_point: Arc<Mutex<Option<PathBuf>>> = Arc::new(Mutex::new(None));

    // gracefully close connection on a non graceful exit
    let interrupted = Arc::new(AtomicBool::new(false));

    let connection_clone = connection.clone();
    let interrupted_clone = interrupted.clone();
    let temp_entry_point_clone = temp_entry_point.clone();
    tokio::task::spawn(async move {
        let _ = signal::ctrl_c().await;
        connection_clone.close(0u32.into(), b"Done");
        interrupted_clone.store(true, Ordering::Relaxed);
        {
            let temp_entry_point = temp_entry_point_clone.lock().await;
            if temp_entry_point.is_some() {
                let res = clean_up_temp(temp_entry_point.as_ref().unwrap()).await;
                if res.is_err() { println!("Error cleaning up temp"); }
            }
        }
    });

    let connection_clone = connection.clone();
    let interrupted_clone = interrupted.clone();
    let temp_entry_point_clone = temp_entry_point.clone();
    tokio::task::spawn(async move {
        let mut sigterm = signal(SignalKind::terminate()).expect("failed to listen to sigterm");
        sigterm.recv().await;
        connection_clone.close(0u32.into(), b"Done");
        interrupted_clone.store(true, Ordering::Relaxed);
        {
            let temp_entry_point = temp_entry_point_clone.lock().await;
            if temp_entry_point.is_some() {
                let res = clean_up_temp(temp_entry_point.as_ref().unwrap()).await;
                if res.is_err() { println!("Error cleaning up temp"); }
            }
        }
    });

    let stream = connection
        .open_bi()
        .await
        .context("opening bi stream for sync request");

    if let Err(e) = stream {
        if !interrupted.load(Ordering::Relaxed) { println!("{}", unroll_anyhow_result(e)); }
        connection.close(0u32.into(), b"Done");
        return Ok(());
    }

    let (mut send, mut recv) = stream.unwrap();

    let target_type = if local_path.is_symlink() { SyncTargetType::Symlink }
    else if local_path.is_dir() { SyncTargetType::Directory }
    else { SyncTargetType::File };

    let checksums = send_sync_request(&mut send, &mut recv, &cmd.remote_path, target_type, cmd.block_size)
        .await
        .context("sending sync request message");
    if let Err(e) = checksums {
        if !interrupted.load(Ordering::Relaxed) { println!("{}", unroll_anyhow_result(e)); }
        connection.close(0u32.into(), b"Done");
        return Ok(());
    }
    let checksums = checksums.unwrap();

    if checksums.is_none() {
        connection.close(0u32.into(), b"Done");
        return Ok(());
    }
    let checksums = checksums.unwrap();

    let local_path_clone = cmd.local_path.clone();
    let remote_path_clone = cmd.remote_path.clone();
    let files_clone = checksums.files.clone();
    let dirs_clone = checksums.directories.clone();

    let temp_entry = tokio::task::spawn_blocking(move || {
        generate_temp_entry_point(&local_path_clone, &remote_path_clone, &files_clone, &dirs_clone)
    })
        .await
        .context("running generate temp entry point")?
        .context("generating temp entry point");

    if let Err(e) = temp_entry {
        if !interrupted.load(Ordering::Relaxed) { println!("{}", unroll_anyhow_result(e)); }
        connection.close(0u32.into(), b"Done");
        return Ok(());
    }
    let temp_entry = temp_entry.unwrap();
    {
        let mut ker = temp_entry_point.lock().await;
        *ker = Some(temp_entry.clone());
    }

    let entry_point_path = Arc::new(temp_entry);
    let remote_target_path = Arc::new(remote_path);
    let local_target_path = Arc::new(local_path);
    let args = Arc::new(cmd);
    let file_syncing_results: Vec<Result<bool>> = futures::stream::iter(checksums.files)
        .map(|checksum| { sync_file(checksum, entry_point_path.clone(), local_target_path.clone(), remote_target_path.clone(), connection.clone(), args.clone()) })
        .buffered(args.concurrent_streams)
        .collect()
        .await;

    let err = file_syncing_results.into_iter().find(|res| res.is_err());
    if let Some(Err(e)) = err {
        if !interrupted.load(Ordering::Relaxed) { println!("{}", unroll_anyhow_result(e)); }
        connection.close(0u32.into(), b"Done");
        return Ok(());
    }

    connection.close(0u32.into(), b"Done");

    // remove the old target, rename the newly synced to the old target

    let res = if local_target_path.is_dir() {
        tokio::fs::remove_dir_all(local_target_path.as_ref()).await
    } else {
        tokio::fs::remove_file(local_target_path.as_ref()).await
    };
    if let Err(e) = res {
        println!("Because of an error during the process of replacing the old target with newly synced one, look for a file/dir \
            whose name starts with the name of the sync target. That file/dir is the newly synced one.");
        println!("Error: {}", unroll_anyhow_result(e.into()));
        return Ok(());
    }

    let res = tokio::fs::rename(entry_point_path.as_ref(), local_target_path.as_ref()).await;
    if let Err(e) = res {
        println!("Because of an error during the process of replacing the old target with newly synced one, look for a file/dir \
            whose name starts with the name of the sync target. That file/dir is the newly synced one.");
        println!("Error: {}", unroll_anyhow_result(e.into()));
        return Ok(());
    }

    if args.timestamp && checksums.directories.len() > 0 {
        let res = update_timestamps_on_dirs(&local_target_path, remote_target_path.as_ref().to_str().unwrap(), &checksums.directories)
            .await
            .context("updating timestamps on directories");
        if let Err(e) = res {
            println!("Error: {}", unroll_anyhow_result(e.into()));
            return Ok(());
        }
    }

    println!("Successfully synced");

    Ok(())
}

async fn sync_file(
    file_meta: FileMeta, entry_point_path: Arc<PathBuf>, local_target_path: Arc<PathBuf>,
    remote_target_path: Arc<PathBuf>, connection: Arc<Connection>, args: Arc<Command>
) -> Result<bool> {
    let local_file = &file_meta.path;

    // both unwraps are safe, because both have been done previously in generate_temp_entry_point
    let local_file = local_file.strip_prefix(remote_target_path.to_str().unwrap()).unwrap();
    let local_file = if local_file.starts_with("/") { local_file.strip_prefix("/").unwrap() } else { local_file };
    let mut local_file_path = entry_point_path.as_ref().clone();
    let mut existing_file_path = local_target_path.as_ref().clone();

    // when the target is just a file, stripping prefix would leave an empty path and
    // pushing an empty path causes it to add a trailing / making it a directory
    if local_file.len() > 0 {
        local_file_path.push(local_file);
        existing_file_path.push(local_file);
    }

    if file_meta.is_symlink {
        // temp dir generates regular files in place of symlinks, which has to be first deleted
        remove_file(local_file_path.as_path()).await.context("deleting temp file that is to be replace with symlink")?;

        let target = file_meta.symlink_target.unwrap();
        let target = Path::new(&target);
        symlink(target, local_file_path)
            .await
            .context("creating symlink")?;
        return Ok(true);
    }

    let local_file_checksums = AsyncFileChecksumIter::new(existing_file_path.to_str().unwrap(), args.block_size).await?;
    let mut file_assembler = FileAssembler::new(local_file_path.to_str().unwrap()).await?;

    if let Some(mut local_file_checksums) = local_file_checksums {
        let naive_check_res = if args.naive { naive_check(&existing_file_path, file_meta.size, file_meta.modified_timestamp).await? } else { false };
        for (block_idx, remote_block_checksum) in file_meta.checksums.iter().enumerate() {
            let local_block_checksum = local_file_checksums
                .next()
                .await
                .context("reading existing block checksum")?;

            // avoiding asking server to transmit the block if we have the block already or if we
            // are only doing naive check
            if (args.naive && naive_check_res) || (local_block_checksum.is_some() && &local_block_checksum.unwrap() == remote_block_checksum) {
                let block = local_file_checksums.get_current_block();
                file_assembler.write_next_block(&block).await?;
            } else {
                let (mut send, mut recv) = connection
                    .open_bi()
                    .await
                    .context("opening bi stream")?;

                let block = send_block_request(&mut send, &mut recv, &file_meta.path, block_idx as u64, args.block_size)
                    .await
                    .context("getting block")?;

                if block.is_none() { return Ok(false); }
                let block = block.unwrap();

                file_assembler
                    .write_next_block(&block.block_data)
                    .await?;
            }
        }
    } else {
        for (block_idx, _) in file_meta.checksums.iter().enumerate() {
            let (mut send, mut recv) = connection
                .open_bi()
                .await
                .context("opening bi stream")?;

            let block = send_block_request(&mut send, &mut recv, &file_meta.path, block_idx as u64, args.block_size)
                .await
                .context("getting block")?;

            if block.is_none() { return Ok(false); }
            let block = block.unwrap();

            file_assembler
                .write_next_block(&block.block_data)
                .await?;
        }
    }

    if args.timestamp { file_assembler.set_modified_timestamp(file_meta.modified_timestamp).await?; }

    #[cfg(unix)]
    {
        if args.permissions && file_meta.permissions.is_some() { file_assembler.set_permissions(file_meta.permissions.unwrap()).await?; }
    }

    Ok(true)
}
