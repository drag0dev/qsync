mod file_meta;
pub mod message;
mod file_checksum_iter;

pub use file_meta::FileMeta;
pub static CHUNK_SIZE: usize = 16 * 1024;
pub use file_checksum_iter::FileChecksumIter;
