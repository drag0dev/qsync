use std::io::SeekFrom;
use super::CHUNK_SIZE;
use anyhow::{Result, Context};
use tokio::{fs::OpenOptions, io::{AsyncReadExt, AsyncSeekExt}};

pub async fn read_block(path: &str, block_idx: u64) -> Result<Vec<u8>> {
    let mut file = OpenOptions::new()
        .read(true)
        .open(&path)
        .await
        .context("opening file block")?;

    let mut buff = vec![0; CHUNK_SIZE];
    file.seek(SeekFrom::Start(CHUNK_SIZE as u64 * block_idx))
        .await
        .context("seeking file")?;

    let n = file.read(&mut buff)
        .await
        .context("reading block")?;
    buff.truncate(n);

    Ok(buff)
}
