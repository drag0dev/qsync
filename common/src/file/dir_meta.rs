use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct DirMeta {
    pub path: String,

    /// timestamp in ms
    pub modified_timestamp: u128,
}

impl DirMeta {
    pub fn new(path: String, modified_timestamp: u128, ) -> Self {
        DirMeta { path, modified_timestamp }
    }
}
