use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum MessageType {
    /// Client sends the dir/file information (paths, checksums)
    SyncRequest,

    /// Server responds to SyncRequest with its own dir/file information (paths, checksums)
    ChecksumsResponse,

    /// Client requests specific block by block index an path
    BlockRequest,

    /// Server responds to BlockRequest with block index, path, and the block data
    BlockData,

    /// Finalizes the transfer
    Complete
}
