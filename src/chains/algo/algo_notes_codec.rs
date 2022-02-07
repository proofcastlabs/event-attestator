use std::str::FromStr;

use base64::{decode as base64_decode, encode as base64_encode};
use derive_more::Deref;
use ethereum_types::Address as EthAddress;

use crate::{
    constants::SAFE_ETH_ADDRESS,
    errors::AppError,
    metadata::metadata_chain_id::MetadataChainId,
    types::{Byte, Bytes, Result},
};

const ALGO_NOTE_MAX_NUM_BYTES: usize = 1000;
const ALGO_NOTE_VERSION_ENCODING_LENGTH: usize = 1;
const ALGO_NOTE_EVM_ADDRESS_ENCODING_LENGTH: usize = 20;
const ALGO_NOTE_METADATA_CHAIN_ID_ENCODING_LENGTH: usize = 4;
const EVM_ALGO_NOTE_ENCODING_LENGTH: usize = ALGO_NOTE_VERSION_ENCODING_LENGTH
    + ALGO_NOTE_METADATA_CHAIN_ID_ENCODING_LENGTH
    + ALGO_NOTE_EVM_ADDRESS_ENCODING_LENGTH;

#[derive(Clone, Debug, Copy, Eq, PartialEq)]
enum AlgoNoteEncodingVersion {
    V0,
}

impl Default for AlgoNoteEncodingVersion {
    fn default() -> Self {
        Self::V0
    }
}

impl AlgoNoteEncodingVersion {
    fn as_byte(&self) -> Byte {
        match self {
            Self::V0 => 0u8,
        }
    }

    fn from_byte(byte: &Byte) -> Result<Self> {
        match byte {
            0u8 => Ok(Self::V0),
            _ => Err(format!("Unrecognized byte for `AlgoNoteEncodingVersion`: 0x{:x}", byte).into()),
        }
    }
}

impl std::fmt::Display for AlgoNoteEncodingVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "0x{}", hex::encode(&vec![self.as_byte()]))
    }
}

#[derive(Clone, Debug, Default, Deref)]
struct AlgoNote(Bytes);

impl AlgoNote {
    pub fn new(bytes: Bytes) -> Result<Self> {
        Self::check_number_of_bytes(bytes).map(Self)
    }

    pub fn check_number_of_bytes(bytes: Bytes) -> Result<Bytes> {
        let num_bytes = bytes.len();
        if num_bytes > ALGO_NOTE_MAX_NUM_BYTES {
            return Err(format!(
                "Cannot encode note, got {} bytes, maximum is {}!",
                num_bytes, ALGO_NOTE_MAX_NUM_BYTES
            )
            .into());
        };
        Ok(bytes)
    }

    pub fn to_base64(&self) -> String {
        base64_encode(self.0.clone())
    }

    fn to_version(&self) -> Result<AlgoNoteEncodingVersion> {
        if self.is_empty() {
            Err("Not enough bytes to get `AlgoNoteEncodingVersion` from AlgoNote!".into())
        } else {
            Ok(AlgoNoteEncodingVersion::from_byte(&self[0])?)
        }
    }

    fn to_metadata_chain_id(&self) -> Result<MetadataChainId> {
        let start_index = ALGO_NOTE_VERSION_ENCODING_LENGTH;
        let end_index = ALGO_NOTE_VERSION_ENCODING_LENGTH + ALGO_NOTE_METADATA_CHAIN_ID_ENCODING_LENGTH;
        if self.len() < end_index {
            Err("Not enough bytes to get `MetadataChainId`from AlgoNote!".into())
        } else {
            MetadataChainId::from_bytes(&self[start_index..end_index])
        }
    }

    fn to_evm_address(&self) -> Result<EthAddress> {
        let start_index = ALGO_NOTE_VERSION_ENCODING_LENGTH + ALGO_NOTE_METADATA_CHAIN_ID_ENCODING_LENGTH;
        let end_index = start_index + ALGO_NOTE_EVM_ADDRESS_ENCODING_LENGTH;
        if self.len() != end_index {
            Err("Not enough bytes to get `EVM ADDRESS` from AlgoNote".into())
        } else {
            Ok(EthAddress::from_slice(&self[start_index..end_index]))
        }
    }

    // FIXME a type for this?
    pub fn decode_for_evm_chain(&self) -> Result<(AlgoNoteEncodingVersion, MetadataChainId, EthAddress)> {
        let length = self.len();
        if length != EVM_ALGO_NOTE_ENCODING_LENGTH {
            info!("✘ Cannot decode AlgoNote into EVM address, defaulting to safe address and interim chain!");
            Ok((
                AlgoNoteEncodingVersion::V0,
                MetadataChainId::InterimChain,
                *SAFE_ETH_ADDRESS,
            ))
        } else {
            Ok((self.to_version()?, self.to_metadata_chain_id()?, self.to_evm_address()?))
        }
    }

    pub fn encode_for_evm_chains(
        version: &AlgoNoteEncodingVersion,
        metadata_chain_id: &MetadataChainId,
        address: &EthAddress,
    ) -> Result<Self> {
        Self::new(
            vec![
                vec![version.as_byte()],
                metadata_chain_id.to_bytes()?,
                address.as_bytes().to_vec(),
            ]
            .concat(),
        )
    }

    pub fn to_bytes(&self) -> Bytes {
        self.0.clone()
    }
}

impl FromStr for AlgoNote {
    type Err = AppError;

    fn from_str(s: &str) -> Result<Self> {
        Self::check_number_of_bytes(base64_decode(s)?).and_then(Self::new)
    }
}

impl std::fmt::Display for AlgoNote {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_base64())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_err_on_too_many_bytes_for_algo_note() {
        const NUM_BYTES: usize = ALGO_NOTE_MAX_NUM_BYTES + 1;
        let bytes = vec![0u8; NUM_BYTES];
        let expected_error = format!(
            "Cannot encode note, got {} bytes, maximum is {}!",
            NUM_BYTES, ALGO_NOTE_MAX_NUM_BYTES
        );
        match AlgoNote::new(bytes) {
            Ok(_) => panic!("Should not have succeeded!"),
            Err(AppError::Custom(error)) => assert_eq!(error, expected_error),
            Err(_) => panic!("Wrong error received!"),
        }
    }

    #[test]
    fn should_encode_and_decode_for_evm_chains() {
        let version = AlgoNoteEncodingVersion::default();
        let address = EthAddress::default();
        let metadata_chain_id = MetadataChainId::default();
        let expected_encoding = "00005fe7f90000000000000000000000000000000000000000";
        let encoding = AlgoNote::encode_for_evm_chains(&version, &metadata_chain_id, &address)
            .unwrap()
            .to_bytes();
        assert_eq!(hex::encode(&encoding), expected_encoding);
        let result = AlgoNote(encoding.clone()).decode_for_evm_chain().unwrap();
        assert_eq!(result.0, version);
        assert_eq!(result.1, metadata_chain_id);
        assert_eq!(result.2, address);
    }

    #[test]
    fn decoding_wrong_length_data_to_evm_should_default_to_safe_address_on_interim_chain() {
        let data = vec![];
        assert_ne!(data.len(), EVM_ALGO_NOTE_ENCODING_LENGTH);
        let result = AlgoNote(data).decode_for_evm_chain().unwrap();
        assert_eq!(result.0, AlgoNoteEncodingVersion::V0);
        assert_eq!(result.1, MetadataChainId::InterimChain);
        assert_eq!(result.2, *SAFE_ETH_ADDRESS);
    }
}
