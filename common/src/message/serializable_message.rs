use anyhow::Result;

pub trait SerializableMessage {
    fn serialize(&self) -> Result<Vec<u8>>;
}
