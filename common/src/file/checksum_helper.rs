use std::{
    collections::VecDeque,
    path::{Path, PathBuf},
    time::SystemTime
};
use anyhow::{anyhow, Context, Result};
use super::{
    FileMeta,
    FileChecksumIter
};

pub fn get_checksums(path_str: &str) -> Result<Vec<FileMeta>> {
    let path = Path::new(path_str);

    // TODO: handle symlinks
    let mut res = Vec::new();
    if path.is_file() {
        let checksums = get_file_checkums(&path_str).context("getting file checksums")?;
        let meta = get_file_meta(&path.into())?;
        let meta = FileMeta::new(path_str.to_owned(), checksums, meta.0, meta.1);
        res.push(meta);
    }
    else if path.is_symlink() {}
    else {
        let mut dirs = VecDeque::new();
        dirs.push_back(path.to_owned());
        while dirs.len() > 0 {
            let mut n = dirs.len();
            while n > 0 {
                let dir_entries = dirs
                    .pop_front()
                    .unwrap()
                    .read_dir()
                    .context("reading directory items")?;
                for entry in dir_entries {
                    let entry = entry.context("reading entry in a directory")?;
                    let entry = entry.path();

                    let entry_item_path = entry.to_str();
                    if entry_item_path.is_none() { return Err(anyhow!("Cannot get directory item path")); }
                    let entry_item_path = entry_item_path.unwrap().to_owned();

                    if entry.is_file() {
                        let checksums = get_file_checkums(&entry_item_path).context("getting file checksums")?;
                        let meta = get_file_meta(&entry)?;
                        let meta = FileMeta::new(entry_item_path, checksums, meta.0, meta.1);
                        res.push(meta);
                    } else if entry.is_symlink() {}
                    else { dirs.push_back(entry); }
                }
                n -= 1;
            }
        }
    }

     Ok(res)
}

fn get_file_meta(file: &PathBuf) -> Result<(u128, u64)> {
    let meta = file.metadata().context("getting file metadata")?;

    let modified_timestamp = meta
        .modified()
        .context("getting accessed timestamp")?
        .duration_since(SystemTime::UNIX_EPOCH)
        .context("calculating timestamp")?
        .as_millis();

    Ok((modified_timestamp, meta.len()))
}

fn get_file_checkums(path: &str) -> Result<Vec<String>> {
    let iter = FileChecksumIter::new(&path)
        .context("instantiating file checksum iter")?;
    iter.into_iter().collect()
}
