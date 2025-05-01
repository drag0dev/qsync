use std::{
    fs::Permissions,
    os::unix::fs::PermissionsExt,
    time::{Duration, SystemTime, UNIX_EPOCH}
};
use anyhow::{Result, Context};
use tokio::{
    fs::{File, OpenOptions},
    io::AsyncWriteExt
};

pub struct FileAssembler {
    file: File
}

impl FileAssembler {
    pub async fn new(file_path: &str) -> Result<Self> {
        let file = OpenOptions::new()
            .write(true)
            .open(file_path)
            .await
            .context("opening file")?;

        Ok(FileAssembler { file })
    }

    pub async fn write_next_block(&mut self, block: &[u8]) -> Result<()> {
        self.file
            .write_all(block)
            .await
            .context("writing a block")?;
        Ok(())
    }

    pub async fn set_modified_timestamp(&mut self, timestamp: u128) -> Result<()> {
        let f = self.file.try_clone().await.context("cloning fd when setting timestamp")?;
        let f = f.into_std().await;
        tokio::task::spawn_blocking(move || {
            let secs = (timestamp / 1000) as u64;
            let sub_millies = (timestamp % 1000) as u32;
            let nanos = sub_millies * 1_000_000;
            let timestamp = UNIX_EPOCH + Duration::new(secs, nanos);
            f.set_modified(SystemTime::from(timestamp)).context("setting modified timestamp")
        }).await??;

        Ok(())
    }

    pub async fn set_permissions(&mut self, permissions: u32) -> Result<()> {
        let f = self.file.try_clone().await.context("cloning fd when setting timestamp")?;
        let f = f.into_std().await;
        tokio::task::spawn_blocking(move || {
            f.set_permissions(Permissions::from_mode(permissions))
        }).await??;

        Ok(())
    }
}
