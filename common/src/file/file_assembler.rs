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
}
