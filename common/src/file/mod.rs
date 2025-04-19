mod file_meta;
mod file_checksum_iter;

pub use file_meta::FileMeta;
pub use file_checksum_iter::FileChecksumIter;
pub static CHUNK_SIZE: usize = 16 * 1024;
