use derive_more::Constructor;
use rust_algorand::AlgorandAddress;
use serde::{Deserialize, Serialize};

use crate::types::{Byte, Bytes, Result};

#[derive(Debug, Default, Clone, Eq, PartialEq, Serialize, Constructor, Deserialize)]
pub struct AlgoUserData {
    addresses: Vec<AlgorandAddress>,

    #[serde(with = "serde_bytes")]
    user_data: Bytes,
}

impl AlgoUserData {
    #[cfg(test)]
    pub fn to_bytes(&self) -> Result<Bytes> {
        Ok(rmp_serde::to_vec(self)?)
    }

    pub fn from_bytes(bytes: &[Byte]) -> Result<Self> {
        rmp_serde::from_slice::<Self>(bytes).map_err(|_| "Could not parse `AlgoUserData` from bytes!".into())
    }

    pub fn to_addresses(&self) -> Vec<AlgorandAddress> {
        self.addresses.clone()
    }

    pub fn to_user_data(&self) -> Bytes {
        self.user_data.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::AppError;

    fn get_sample_address() -> AlgorandAddress {
        AlgorandAddress::from_bytes(&[0x33; 32]).unwrap()
    }

    fn get_sample_bytes() -> Bytes {
        hex::decode("9291c4203333333333333333333333333333333333333333333333333333333333333333c403c0ffee").unwrap()
    }

    fn get_sample_user_data() -> Bytes {
        hex::decode("c0ffee").unwrap()
    }

    #[test]
    fn should_encode_algo_user_data_correctly() {
        let user_data = get_sample_user_data();
        let addresses = vec![get_sample_address()];
        let algo_user_data = AlgoUserData::new(addresses, user_data);
        let expected_result = get_sample_bytes();
        let result = algo_user_data.to_bytes().unwrap();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_get_algo_user_data_from_bytes() {
        let bytes = get_sample_bytes();
        let result = AlgoUserData::from_bytes(&bytes).unwrap();
        let expected_result = AlgoUserData::new(vec![get_sample_address()], get_sample_user_data());
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_fail_to_get_algo_user_data_from_bytes() {
        let bytes = hex::decode("c0ffee").unwrap();
        let expected_error = "Could not parse `AlgoUserData` from bytes!";
        match AlgoUserData::from_bytes(&bytes) {
            Ok(_) => panic!("Should not have succeeded!"),
            Err(AppError::Custom(error)) => assert_eq!(error, expected_error),
            Err(_) => panic!("Wrong error received!"),
        }
    }
}
