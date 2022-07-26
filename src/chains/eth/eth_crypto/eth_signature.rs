use std::str::FromStr;

use derive_more::{Constructor, Deref, DerefMut};
use ethereum_types::{Address as EthAddress, H256};
use web3::signing::recover;

use crate::{errors::AppError, types::Result, utils::strip_hex_prefix};

const ETH_SIGNATURE_NUM_BYTES: usize = 65;

#[derive(Clone, Debug, Eq, PartialEq, Deref, DerefMut, Constructor)]
pub struct EthSignature(pub [u8; 65]);

impl EthSignature {
    pub fn set_recovery_param(self) -> Self {
        // NOTE: Eth recovery params are different from ecdsa ones. See here for more info:
        // https://bitcoin.stackexchange.com/questions/38351/ecdsa-v-r-s-what-is-v
        let mut mutable_self = self;
        mutable_self[64] = if mutable_self[64] == 1 { 0x1c } else { 0x1b };
        mutable_self
    }

    fn get_ecdsa_recovery_param(&self) -> u8 {
        match self[64] {
            0x1c => 1,
            _ => 0,
        }
    }

    pub fn recover_signer_address(&self, hash: &H256) -> Result<EthAddress> {
        Ok(recover(
            hash.as_bytes(),
            &self[..64],
            self.get_ecdsa_recovery_param().into(),
        )?)
    }
}

impl FromStr for EthSignature {
    type Err = AppError;

    fn from_str(s: &str) -> Result<Self> {
        let bytes = hex::decode(strip_hex_prefix(s))?;
        Ok(Self::new(bytes.clone().try_into().map_err(|_| {
            AppError::Custom(format!(
                "Wrong number of bytes for `EthSignature`. Got {}, expected {}!",
                bytes.len(),
                ETH_SIGNATURE_NUM_BYTES
            ))
        })?))
    }
}

impl TryFrom<&str> for EthSignature {
    type Error = AppError;

    fn try_from(s: &str) -> Result<Self> {
        EthSignature::from_str(s)
    }
}

impl std::fmt::Display for EthSignature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", hex::encode(&self.0))
    }
}

#[cfg(test)]
use crate::types::Bytes;

#[cfg(test)]
impl EthSignature {
    pub fn random() -> Result<Self> {
        let random_bytes = (0..ETH_SIGNATURE_NUM_BYTES)
            .map(|_| rand::random::<u8>())
            .collect::<Bytes>();
        Self::from_str(&hex::encode(&random_bytes))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn should_get_eth_signature_from_st() {
        let result = EthSignature::from_str("0xda1a3b8f1bb8c0964b15785b5408ca3dfe35ed512d860d03bc543656e0c8f2a72c550b23a15b4c6624b3625217380ce1849e85710278ddd4aaee5d8b4f26d1521c");
        assert!(result.is_ok());
    }

    #[test]
    fn should_err_if_not_enough_bytes_for_eth_signature() {
        let expected_error = "Wrong number of bytes for `EthSignature`. Got 64, expected 65!".to_string();
        match EthSignature::from_str("0xda1a3b8f1bb8c0964b15785b5408ca3dfe35ed512d860d03bc543656e0c8f2a72c550b23a15b4c6624b3625217380ce1849e85710278ddd4aaee5d8b4f26d152") {
            Ok(_) => panic!("Should not have succeeded!"),
            Err(AppError::Custom(error)) => assert_eq!(error, expected_error),
            Err(_) => panic!("Wrong error received!"),
        }
    }

    #[test]
    fn try_into_should_work_for_eth_signature() {
        let result: Result<EthSignature> = "0xda1a3b8f1bb8c0964b15785b5408ca3dfe35ed512d860d03bc543656e0c8f2a72c550b23a15b4c6624b3625217380ce1849e85710278ddd4aaee5d8b4f26d1521c".try_into();
        assert!(result.is_ok());
    }
}
