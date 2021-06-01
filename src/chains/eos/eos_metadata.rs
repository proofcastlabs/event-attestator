use derive_more::Constructor;
use crate::types::Bytes;
use eos_chain::{NumBytes, Read, SerializeData, Write};

#[derive(Clone, Debug, Read, Write, NumBytes, PartialEq, Default, Constructor)]
#[eosio_core_root_path = "eos_chain"]
pub struct EosMetadata {
    pub version: Bytes,
    pub metadata_chain_id: Bytes,
    pub origin_address: Bytes,
    pub user_data: Bytes,
}

impl SerializeData for EosMetadata {}

impl EosMetadata {
    pub fn to_bytes(&self) -> crate::types::Result<Bytes> {
        Ok(self.to_serialize_data()?)
    }
}
