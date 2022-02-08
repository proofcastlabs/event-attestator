use std::str::FromStr;

use base64::{decode as base64_decode, encode as base64_encode};
use derive_more::{Constructor, Deref};
use ethereum_types::Address as EthAddress;
use rmp_serde;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

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
const MINIMUM_EVM_ALGO_NOTE_ENCODING_LENGTH: usize = ALGO_NOTE_VERSION_ENCODING_LENGTH
    + ALGO_NOTE_METADATA_CHAIN_ID_ENCODING_LENGTH
    + ALGO_NOTE_EVM_ADDRESS_ENCODING_LENGTH;

#[derive(Clone, Debug, Default, Eq, PartialEq, Constructor)]
pub struct AlgoNoteMetadata {
    pub version: AlgoNoteMetadataVersion,
    pub destination_chain_id: MetadataChainId,
    pub destination_address: String,
    pub user_data: Bytes,
}

impl AlgoNoteMetadata {
    pub fn from_bytes(bytes: &[Byte]) -> Result<Self> {
        Self::from_msg_pack(&AlgoMetadataMsgPack::from_bytes(bytes)?)
    }

    pub fn to_bytes(&self) -> Result<Bytes> {
        self.to_msg_pack()?.to_bytes()
    }

    pub fn from_msg_pack(msg_pack: &AlgoMetadataMsgPack) -> Result<Self> {
        Ok(Self {
            version: AlgoNoteMetadataVersion::from_byte(&msg_pack.version)?,
            destination_chain_id: MetadataChainId::from_bytes(&msg_pack.destination_chain_id)?,
            destination_address: msg_pack.destination_address.clone(),
            user_data: match &msg_pack.user_data {
                Some(bytes) => bytes.clone(),
                None => vec![],
            },
        })
    }

    pub fn to_msg_pack(&self) -> Result<AlgoMetadataMsgPack> {
        Ok(AlgoMetadataMsgPack {
            version: self.version.as_byte(),
            destination_address: self.destination_address.clone(),
            destination_chain_id: self.destination_chain_id.to_bytes()?,
            user_data: if self.user_data.is_empty() {
                None
            } else {
                Some(self.user_data.clone())
            },
        })
    }
}

#[skip_serializing_none]
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize, Constructor)]
pub struct AlgoMetadataMsgPack {
    version: Byte,
    #[serde(rename = "chainId")]
    destination_chain_id: Bytes,
    destination_address: String,
    #[serde(rename = "userData")]
    user_data: Option<Bytes>,
}

impl AlgoMetadataMsgPack {
    pub fn to_bytes(&self) -> Result<Bytes> {
        Ok(rmp_serde::to_vec(self)?)
    }

    pub fn from_bytes(bytes: &[Byte]) -> Result<Self> {
        Ok(rmp_serde::from_read_ref(bytes)?)
    }
}

#[derive(Clone, Debug, Copy, Eq, PartialEq)]
pub enum AlgoNoteMetadataVersion {
    V0,
}

impl Default for AlgoNoteMetadataVersion {
    fn default() -> Self {
        Self::V0
    }
}

impl AlgoNoteMetadataVersion {
    fn as_byte(&self) -> Byte {
        match self {
            Self::V0 => 0u8,
        }
    }

    fn from_byte(byte: &Byte) -> Result<Self> {
        match byte {
            0u8 => Ok(Self::V0),
            _ => Err(format!("Unrecognized byte for `AlgoNoteMetadataVersion`: 0x{:x}", byte).into()),
        }
    }
}

impl std::fmt::Display for AlgoNoteMetadataVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "0x{}", hex::encode(&vec![self.as_byte()]))
    }
}

/*
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

    fn to_version(&self) -> Result<AlgoNoteMetadataVersion> {
        if self.is_empty() {
            Err("Not enough bytes to get `AlgoNoteMetadataVersion` from AlgoNote!".into())
        } else {
            Ok(AlgoNoteMetadataVersion::from_byte(&self[0])?)
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
        if self.len() < end_index {
            Err("Not enough bytes to get `EVM ADDRESS` from AlgoNote".into())
        } else {
            Ok(EthAddress::from_slice(&self[start_index..end_index]))
        }
    }

    fn to_user_data(&self) -> Bytes {
        let length = self.len();
        if length <= MINIMUM_EVM_ALGO_NOTE_ENCODING_LENGTH {
            info!("✘ No user data included in note");
            vec![]
        } else {
            self[MINIMUM_EVM_ALGO_NOTE_ENCODING_LENGTH..].to_vec()
        }
    }

    // FIXME a type for this?
    pub fn decode_for_evm_chain(&self) -> Result<(AlgoNoteMetadataVersion, MetadataChainId, EthAddress, Bytes)> {
        let length = self.len();
        if length < MINIMUM_EVM_ALGO_NOTE_ENCODING_LENGTH {
            info!("✘ Cannot decode AlgoNote into EVM destination_address, defaulting to safe destination_address and interim chain!");
            Ok((
                AlgoNoteMetadataVersion::V0,
                MetadataChainId::InterimChain,
                *SAFE_ETH_ADDRESS,
                vec![],
            ))
        } else {
            Ok((
                self.to_version()?,
                self.to_metadata_chain_id()?,
                self.to_evm_address()?,
                self.to_user_data(),
            ))
        }
    }

    pub fn encode_for_evm_chains(
        version: &AlgoNoteMetadataVersion,
        destination_chain_id: &MetadataChainId,
        destination_address: &EthAddress,
        user_data: &[Byte],
    ) -> Result<Self> {
        Self::new(
            vec![
                vec![version.as_byte()],
                destination_chain_id.to_bytes()?,
                destination_address.as_bytes().to_vec(),
                user_data.to_vec(),
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
*/

#[cfg(test)]
mod tests {
    use super::*;

    /*
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
        let version = AlgoNoteMetadataVersion::default();
        let destination_address = EthAddress::default();
        let destination_chain_id = MetadataChainId::default();
        let user_data = vec![0xc0, 0xff, 0xee];
        let expected_encoding = "00ffffffff0000000000000000000000000000000000000000c0ffee";
        let encoding = AlgoNote::encode_for_evm_chains(&version, &destination_chain_id, &destination_address, &user_data)
            .unwrap()
            .to_bytes();
        assert_eq!(hex::encode(&encoding), expected_encoding);
        let result = AlgoNote(encoding.clone()).decode_for_evm_chain().unwrap();
        assert_eq!(result.0, version);
        assert_eq!(result.1, destination_chain_id);
        assert_eq!(result.2, destination_address);
        assert_eq!(result.3, user_data);
    }

    #[test]
    fn decoding_wrong_length_data_to_evm_should_default_to_safe_address_on_interim_chain() {
        let data = vec![];
        assert_ne!(data.len(), MINIMUM_EVM_ALGO_NOTE_ENCODING_LENGTH);
        let result = AlgoNote(data.clone()).decode_for_evm_chain().unwrap();
        assert_eq!(result.0, AlgoNoteMetadataVersion::V0);
        assert_eq!(result.1, MetadataChainId::InterimChain);
        assert_eq!(result.2, *SAFE_ETH_ADDRESS);
        assert_eq!(result.3, data);
    }
    */

    #[test]
    fn should_serde_algo_metadata_to_bytes() {
        let user_data = vec![0xc0, 0xff, 0xee];
        let destination_address = "someaddress".to_string();
        let metadata = AlgoNoteMetadata::new(
            AlgoNoteMetadataVersion::V0,
            MetadataChainId::EthereumMainnet,
            destination_address,
            user_data,
        );
        let bytes = metadata.to_bytes().unwrap();
        let expected_bytes = "940094005fcce7ccf9ab736f6d656164647265737393ccc0ccffccee";
        assert_eq!(hex::encode(&bytes), expected_bytes);
        let result = AlgoNoteMetadata::from_bytes(&bytes).unwrap();
        assert_eq!(result, metadata);
    }
}
