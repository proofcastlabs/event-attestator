use common::types::{Byte, Bytes, Result};
use derive_more::Constructor;
use rust_algorand::AlgorandAddress;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Eq, PartialEq, Serialize, Constructor, Deserialize)]
pub struct AlgoUserData {
    #[serde(with = "serde_bytes")]
    user_data: Bytes,

    addresses: Vec<AlgorandAddress>,

    app_ids: Vec<u64>,

    asset_ids: Vec<u64>,
}

impl AlgoUserData {
    pub fn from_bytes(bytes: &[Byte]) -> Result<Self> {
        info!("✔ Parsing `AlgoUserData` from bytes...");
        Ok(rmp_serde::from_slice::<Self>(bytes)?)
    }

    pub fn to_addresses(&self) -> Vec<AlgorandAddress> {
        self.addresses.clone()
    }

    pub fn to_user_data(&self) -> Bytes {
        self.user_data.clone()
    }

    pub fn to_asset_ids(&self) -> Vec<u64> {
        self.asset_ids.clone()
    }

    pub fn to_app_ids(&self) -> Vec<u64> {
        self.app_ids.clone()
    }
}

#[cfg(test)]
impl AlgoUserData {
    #[allow(dead_code)]
    pub fn to_bytes(&self) -> Result<Bytes> {
        Ok(rmp_serde::to_vec(self)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_decode_msg_packed_algo_user_data() {
        let bytes = hex::decode("94c44b95c4208cf5862026483a9b12b349d126811ed1cebcb759b63556130be48ce3e062fc5bc40800000000248759b7c40800000000000003e8c408000000002dc37915c408000000000000000191c4208cf5862026483a9b12b349d126811ed1cebcb759b63556130be48ce3e062fc5b9092ce248759b7ce2dc37915").unwrap();
        let result = AlgoUserData::from_bytes(&bytes);
        assert!(result.is_ok());
    }
}
