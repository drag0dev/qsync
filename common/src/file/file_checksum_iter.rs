use std::{
    fs::File,
    io::{Read, Seek}
};
use anyhow::{Context, Result};

pub struct FileChecksumIter {
    file: File,
    buffer: Vec<u8>,
}

impl FileChecksumIter {
    pub fn new(path: &str, block_size: usize) -> Result<Self> {
        let mut file = File::open(path).context("opening file in checksum iter")?;
        file.rewind().context("rewinding file in checksum iter")?;
        Ok(FileChecksumIter { file, buffer: vec![0; block_size] })
    }
}

impl Iterator for FileChecksumIter {
    type Item = Result<String>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.file.read(&mut self.buffer) {
            Ok(0) => None,
            Ok(n) => {
                let hash = blake3::hash(&self.buffer[..n]);
                Some(Ok(hash.to_string()))
            }
            Err (e) => Some(Err(e.into()))
        }
    }
}
