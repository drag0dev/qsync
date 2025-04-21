use std::{
    fs::File,
    io::{Read, Seek}
};
use anyhow::{Context, Result};
use super::CHUNK_SIZE;

pub struct FileChecksumIter {
    file: File,
    buffer: [u8; CHUNK_SIZE],
}

impl FileChecksumIter {
    pub fn new(path: &str) -> Result<Self> {
        let mut file = File::open(path).context("opening file in checksum iter")?;
        file.rewind().context("rewinding file in checksum iter")?;
        Ok(FileChecksumIter { file, buffer: [0; CHUNK_SIZE] })
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
