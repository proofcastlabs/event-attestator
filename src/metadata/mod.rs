pub(crate) mod metadata_chain_id;
pub(crate) mod metadata_origin_address;
pub(crate) mod metadata_protocol_id;
pub(crate) mod metadata_traits;
pub(crate) mod metadata_version;
pub(crate) mod test_utils;

use ethabi::{encode as eth_abi_encode, Token as EthAbiToken};
use ethereum_types::Address as EthAddress;

use crate::{
    chains::eos::eos_metadata::EosMetadata,
    metadata::{
        metadata_chain_id::MetadataChainId,
        metadata_origin_address::MetadataOriginAddress,
        metadata_protocol_id::MetadataProtocolId,
        metadata_version::MetadataVersion,
    },
    types::{Byte, Bytes, NoneError, Result},
};

/// Metadata V1 Specification per @bertani:
/// [
///     uint8 versionByte,
///     bytes userData,
///     bytes4 originProtocol <bytes1 originProtocolId + bytes3 keccak256(originChainId)[:3]>,
///     origin sender
/// ]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Metadata {
    pub version: MetadataVersion,
    pub user_data: Bytes,
    pub origin_chain_id: MetadataChainId,
    pub origin_address: MetadataOriginAddress,
    pub destination_chain_id: Option<MetadataChainId>,
    pub destination_address: Option<MetadataOriginAddress>,
    pub protocol_options: Option<Bytes>,
    pub protocol_receipt: Option<Bytes>,
}

impl Metadata {
    pub fn get_destination_chain_id(&self) -> Result<MetadataChainId> {
        match self.version {
            MetadataVersion::V1 => Err("Cannot get destination chain ID from v1 metadata!".into()),
            MetadataVersion::V2 => self
                .destination_chain_id
                .ok_or(NoneError("Error getting destinaction chain ID!")),
        }
    }

    pub fn new(user_data: &[Byte], origin_address: &MetadataOriginAddress) -> Self {
        Self::new_v1(user_data, origin_address)
    }

    fn new_v1(user_data: &[Byte], origin_address: &MetadataOriginAddress) -> Self {
        Self {
            version: MetadataVersion::V1,
            user_data: user_data.to_vec(),
            origin_address: origin_address.clone(),
            origin_chain_id: origin_address.metadata_chain_id,
            destination_chain_id: None,
            destination_address: None,
            protocol_options: None,
            protocol_receipt: None,
        }
    }

    pub fn new_v2(
        user_data: &[Byte],
        origin_address: &MetadataOriginAddress,
        destination_chain_id: &MetadataChainId,
        destination_address: &MetadataOriginAddress,
        protocol_options: Option<Bytes>,
        protocol_receipt: Option<Bytes>,
    ) -> Self {
        Self {
            protocol_options,
            protocol_receipt,
            version: MetadataVersion::V2,
            user_data: user_data.to_vec(),
            origin_address: origin_address.clone(),
            origin_chain_id: origin_address.metadata_chain_id,
            destination_chain_id: Some(*destination_chain_id),
            destination_address: Some(destination_address.clone()),
        }
    }

    fn to_bytes_for_eth_v1(&self) -> Result<Bytes> {
        Ok(eth_abi_encode(&[
            EthAbiToken::FixedBytes(self.version.to_bytes()),
            EthAbiToken::Bytes(self.user_data.clone()),
            EthAbiToken::FixedBytes(self.origin_chain_id.to_bytes()?),
            match self.origin_address.metadata_chain_id.to_protocol_id() {
                MetadataProtocolId::Ethereum => {
                    EthAbiToken::Address(EthAddress::from_slice(&self.origin_address.to_bytes()?))
                },
                MetadataProtocolId::Eos | MetadataProtocolId::Bitcoin => {
                    EthAbiToken::Bytes(self.origin_address.to_bytes()?)
                },
            },
        ]))
    }

    fn to_bytes_for_eth_v2(&self) -> Result<Bytes> {
        Ok(eth_abi_encode(&[
            EthAbiToken::FixedBytes(self.version.to_bytes()),
            EthAbiToken::Bytes(self.user_data.clone()),
            EthAbiToken::FixedBytes(self.origin_chain_id.to_bytes()?),
            match self.origin_address.metadata_chain_id.to_protocol_id() {
                MetadataProtocolId::Ethereum => {
                    EthAbiToken::Address(EthAddress::from_slice(&self.origin_address.to_bytes()?))
                },
                MetadataProtocolId::Eos | MetadataProtocolId::Bitcoin => {
                    EthAbiToken::Bytes(self.origin_address.to_bytes()?)
                },
            },
            EthAbiToken::FixedBytes(self.get_destination_chain_id()?.to_bytes()?),
            EthAbiToken::Address(EthAddress::from_slice(&match &self.destination_address {
                Some(address) => address.to_bytes(),
                None => Err("No `destination_address` in metadata!".into()),
            }?)),
            EthAbiToken::Bytes(match &self.protocol_options {
                Some(bytes) => bytes.to_vec(),
                None => vec![],
            }),
            EthAbiToken::Bytes(match &self.protocol_receipt {
                Some(bytes) => bytes.to_vec(),
                None => vec![],
            }),
        ]))
    }

    fn to_bytes_for_eth(&self) -> Result<Bytes> {
        match self.version {
            MetadataVersion::V1 => self.to_bytes_for_eth_v1(),
            MetadataVersion::V2 => self.to_bytes_for_eth_v2(),
        }
    }

    fn to_bytes_for_eos(&self) -> Result<Bytes> {
        EosMetadata::new(
            self.version.to_byte(),
            self.user_data.clone(),
            self.origin_chain_id.to_bytes()?,
            format!("0x{}", hex::encode(self.origin_address.to_bytes()?)),
        )
        .to_bytes()
    }

    pub fn to_bytes_for_protocol(&self, destination_protocol: &MetadataProtocolId) -> Result<Bytes> {
        match destination_protocol {
            MetadataProtocolId::Eos => self.to_bytes_for_eos(),
            MetadataProtocolId::Ethereum => self.to_bytes_for_eth(),
            MetadataProtocolId::Bitcoin => Err("Encoding metadata for Bitcoin is not implemented!".into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metadata::test_utils::{get_sample_eth_metadata, get_sample_eth_metadata_v2};

    #[test]
    fn should_encode_eth_metadata_for_eos() {
        let metadata = get_sample_eth_metadata();
        let bytes = metadata.to_bytes_for_eos().unwrap();
        let hex_encoded_bytes = hex::encode(&bytes);
        let expected_hex_encode_bytes = "0103c0ffee04005fe7f92a307835613062353464356463313765306161646333383364326462343362306130643365303239633463";
        assert_eq!(hex_encoded_bytes, expected_hex_encode_bytes);
    }

    #[test]
    fn should_encode_v2_metadata_for_eth() {
        let metadata = get_sample_eth_metadata_v2();
        let result = hex::encode(metadata.to_bytes_for_eth().unwrap());
        let expected_result = "0200000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000010000f3436800000000000000000000000000000000000000000000000000000000000000000000000000000000fedfe2616eb3661cb8fed2782f5f0cc91d59dcac0069c32200000000000000000000000000000000000000000000000000000000000000000000000000000000edb86cd455ef3ca43f0e227e00469c3bdfa40628000000000000000000000000000000000000000000000000000000000000014000000000000000000000000000000000000000000000000000000000000001600000000000000000000000000000000000000000000000000000000000000003d3caff000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000";
        assert_eq!(result, expected_result);
    }
}
