use derive_more::Constructor;
use rmp_serde;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::{
    metadata::metadata_chain_id::MetadataChainId,
    safe_addresses::SAFE_ETH_ADDRESS_STR,
    types::{Byte, Bytes, Result},
    utils::strip_hex_prefix,
};

#[derive(Clone, Debug, Eq, PartialEq, Constructor)]
pub struct AlgoNoteMetadata {
    pub version: AlgoNoteMetadataVersion,
    pub destination_chain_id: MetadataChainId,
    pub destination_address: String,
    pub user_data: Bytes,
}

impl Default for AlgoNoteMetadata {
    fn default() -> Self {
        // NOTE: We default to the safe ETH address and the interim chain ID. This default is then
        // used when a user omits to provide their own encoded note in the Algo tx.
        Self {
            user_data: vec![],
            version: AlgoNoteMetadataVersion::default(),
            destination_chain_id: MetadataChainId::InterimChain,
            destination_address: SAFE_ETH_ADDRESS_STR.to_string(),
        }
    }
}

impl AlgoNoteMetadata {
    fn from_bytes(bytes: &[Byte]) -> Result<Self> {
        Self::from_msg_pack(&AlgoMetadataMsgPack::from_bytes(bytes)?)
    }

    pub fn from_bytes_or_default(bytes: &[Byte]) -> Self {
        // NOTE: So here we default to the interim chain & safe ETH address should reading the
        // metadata from bytes fail. See note above.
        Self::from_bytes(bytes).unwrap_or_default()
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

/// Algo Note Metadata Encoder
///
/// Encodes the Algorand note metadata required to make a pToken redeem asset transfer
/// transaction.
pub fn encode_algo_note_metadata(
    destination_chain_id: &str,
    destination_address: &str,
    user_data: &str,
) -> Result<String> {
    Ok(format!(
        "0x{}",
        hex::encode(
            AlgoNoteMetadata::new(
                AlgoNoteMetadataVersion::V0,
                MetadataChainId::from_bytes(&hex::decode(strip_hex_prefix(destination_chain_id))?)?,
                destination_address.to_string(),
                hex::decode(strip_hex_prefix(user_data))?
            )
            .to_bytes()?
        )
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_encode_algo_note_metadata() {
        let expected_result = "0x940094ccffccffccffccffab736f6d656164647265737393ccc0ccffccee";
        let result = encode_algo_note_metadata("0xffffffff", "someaddress", "0xc0ffee").unwrap();
        assert_eq!(result, expected_result);
    }

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

    #[test]
    fn should_revert_to_default_if_cannot_decode_bytes() {
        let bytes = vec![];
        let result = AlgoNoteMetadata::from_bytes_or_default(&bytes);
        assert_eq!(result, AlgoNoteMetadata::default());
    }
}
