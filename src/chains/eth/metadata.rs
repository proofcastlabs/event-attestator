#![allow(dead_code)] // TODO rm!
use std::{
    fmt,
    str::FromStr,
};
use crate::{
    types::{
        Byte,
        Bytes,
        Result,
    },
    errors::AppError,
    utils::decode_hex_with_err_msg,
    btc_on_eth::btc::btc_types::MintingParamStruct as BtcOnEthMintingParamStruct,
};
use bitcoin::{
    hashes::sha256d,
    util::address::Address as BtcAddress,
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EthMetadataVersion {
    V1,
}

impl EthMetadataVersion {
    pub fn as_byte(&self) -> Byte {
        match self {
            EthMetadataVersion::V1 => 0x01,
        }
    }

    pub fn from_byte(byte: &Byte) -> Result<Self> {
        match byte {
            1u8 => Ok(EthMetadataVersion::V1),
            _ => Err(AppError::Custom(format!("✘ Unrecognized version byte for `EthMetadataVersion`: {:?}", byte)))
        }

    }
}

impl fmt::Display for EthMetadataVersion {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            EthMetadataVersion::V1 => write!(f, "1"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_get_metadata_v1_byte() {
        let expected_result = 0x01;
        let metadata_version = EthMetadataVersion::V1;
        let result = metadata_version.as_byte();
        assert_eq!(result ,expected_result);
    }

    #[test]
    fn should_fail_to_get_metadata_version_correctly() {
        let byte = 255u8;
        let expected_error = format!("✘ Unrecognized version byte for `EthMetadataVersion`: {:?}", byte);
        match EthMetadataVersion::from_byte(&byte) {
            Err(AppError::Custom(err)) => assert_eq!(err, expected_error),
            Err(err) => panic!("Wrong error received: {}", err),
            Ok(_) => panic!("Should not have succeeded!"),
        }
    }
}
