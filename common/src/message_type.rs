use deku::{DekuRead, DekuWrite};

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(id_type = "u8")]
pub enum MessageType {
    /// Client sends the dir/file information (paths, sizes, checksums)
    #[deku(id = 0x01)]
    SyncRequest,

    /// Server responds to SyncRequest with its own dir/file information (paths, sizes, checksums)
    #[deku(id = 0x02)]
    ChecksumsResponse,

    /// Client requests specific block by block index an path
    #[deku(id = 0x03)]
    BlockRequest,

    /// Server responds to BlockRequest with block index, path, and the block data
    #[deku(id = 0x04)]
    BlockData,

    /// Finalizes the transfer
    #[deku(id = 0x05)]
    Complete
}
