use std::{
    collections::VecDeque, os::unix::fs::MetadataExt,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc
    },
    time::SystemTime
};
use anyhow::{anyhow, Context, Result};
use super::{
    DirMeta, FileChecksumIter, FileMeta
};

pub fn get_checksums(path_str: &str, block_size: usize, stop_signal: Arc<AtomicBool>) -> Result<(Vec<FileMeta>, Vec<DirMeta>)> {
    let path = Path::new(path_str);

    let mut dir_meta = Vec::new();

    // TODO: handle symlinks first, because both dir and file can be a symlink
    let mut file_meta = Vec::new();
    if path.is_file() {
        let checksums = get_file_checkums(&path_str, block_size, stop_signal).context("getting file checksums")?;
        let meta = get_file_meta(&path.into())?;
        let meta = FileMeta::new(path_str.to_owned(), checksums, meta.0, meta.1, meta.2);
        file_meta.push(meta);
    }
    else if path.is_symlink() {}
    else {
        let mut dirs = VecDeque::new();
        dirs.push_back(path.to_owned());
        while dirs.len() > 0 {
            let mut n = dirs.len();
            while n > 0 {
                let dir = dirs
                    .pop_front()
                    .unwrap();

                let dir_entries = (&dir)
                    .read_dir()
                    .context("reading directory items")?;
                for entry in dir_entries {
                    let entry = entry.context("reading entry in a directory")?;
                    let entry = entry.path();

                    let entry_item_path = entry.to_str();
                    if entry_item_path.is_none() { return Err(anyhow!("Cannot get directory item path")); }
                    let entry_item_path = entry_item_path.unwrap().to_owned();

                    if entry.is_file() {
                        let checksums = get_file_checkums(&entry_item_path, block_size, stop_signal.clone()).context("getting file checksums")?;
                        let meta = get_file_meta(&entry)?;
                        let meta = FileMeta::new(entry_item_path, checksums, meta.0, meta.1, meta.2);
                        file_meta.push(meta);
                    } else if entry.is_symlink() {}
                    else { dirs.push_back(entry); }
                }

                let dir_timestamp = get_file_meta(&dir).context("getting dir meta")?.0;
                let curr_dir_meta = DirMeta::new(dir.to_str().unwrap().to_owned(), dir_timestamp);
                dir_meta.push(curr_dir_meta);
                n -= 1;
            }
        }
    }

     Ok((file_meta, dir_meta))
}

fn get_file_meta(file: &PathBuf) -> Result<(u128, u64, Option<u32>)> {
    let meta = file.metadata().context("getting file metadata")?;

    let modified_timestamp = meta
        .modified()
        .context("getting accessed timestamp")?
        .duration_since(SystemTime::UNIX_EPOCH)
        .context("calculating timestamp")?
        .as_millis();

    let permissions = {
        #[cfg(unix)]
        { Some(meta.mode()) }

        #[cfg(windows)]
        { None }
    };

    Ok((modified_timestamp, meta.len(), permissions))
}

fn get_file_checkums(path: &str, block_size: usize, stop_signal: Arc<AtomicBool>) -> Result<Vec<String>> {
    let iter = FileChecksumIter::new(&path, block_size)
        .context("instantiating file checksum iter")?;
    iter.into_iter().take_while(|_| !stop_signal.load(Ordering::Relaxed)).collect()
}
