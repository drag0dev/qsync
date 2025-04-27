use std::fmt::{self, Display, Formatter};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum MessageType {
    /// Client sends the dir/file path
    SyncRequest,

    /// Server responds to SyncRequest with its own dir/file information (paths, checksums)
    ChecksumsResponse,

    /// Client requests specific block by block index an path
    BlockRequest,

    /// Server responds to BlockRequest with block index, path, and the block data
    BlockData,

    /// Server responds to BlockRequest with block index, path, and the block data
    Error,
}

impl Display for MessageType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::SyncRequest => write!(f, "Sync Request"),
            Self::ChecksumsResponse => write!(f, "Checksums Response"),
            Self::BlockRequest => write!(f, "Block Request"),
            Self::BlockData => write!(f, "Block Data"),
            Self::Error => write!(f, "Error"),
        }
    }
}
