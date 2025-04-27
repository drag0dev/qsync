use std::io::ErrorKind;

use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncSeekExt}
};
use anyhow::{Context, Result};

pub struct AsyncFileChecksumIter {
    file: File,
    buffer: Vec<u8>,
    loaded_data: usize,
}

impl AsyncFileChecksumIter {
    pub async fn new(path: &str, block_size: usize) -> Result<Option<Self>> {
        let file = File::open(path)
            .await;

        let mut file = match file {
            Ok(f) => f,
            Err(e) => {
                if e.kind() == ErrorKind::NotFound { return Ok(None); }
                else {
                    let e = Err(e).context("opening file in async checksum iter");
                    return e;
                }
            }
        };

        file.rewind()
            .await
            .context("rewinding file in checksum iter")?;

        Ok(Some(AsyncFileChecksumIter { file, buffer: vec![0; block_size], loaded_data: 0}))
    }

    pub fn get_current_block<'a>(&'a self) -> &'a[u8] {
        &self.buffer[..self.loaded_data]
    }

    pub async fn next(&mut self) -> Result<Option<String>> {
        match self.file.read(&mut self.buffer).await {
            Ok(0) => {
                self.loaded_data = 0;
                Ok(None)
            }
            Ok(n) => {
                self.loaded_data = n;
                let hash = blake3::hash(&self.buffer[..n]);
                Ok(Some(hash.to_string()))
            }
            Err (e) => Err(e.into())
        }
    }
}
