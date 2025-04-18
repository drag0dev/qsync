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

    /// Finalizes the transfer
    Complete
}
