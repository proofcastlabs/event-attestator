#![allow(dead_code)] // FIXME Rm!

pub(crate) mod blockchain_chain_id;
pub(crate) mod blockchain_protocol_id;
pub(crate) mod metadata_origin_address;
pub(crate) mod metadata_version;
pub(crate) mod test_utils;

use ethabi::{decode as eth_abi_decode, encode as eth_abi_encode, ParamType as EthAbiParamType, Token as EthAbiToken};
use ethereum_types::Address as EthAddress;

use crate::{
    metadata::{
        blockchain_chain_id::BlockchainChainId,
        blockchain_protocol_id::BlockchainProtocolId,
        metadata_origin_address::MetadataOriginAddress,
        metadata_version::MetadataVersion,
    },
    types::{Byte, Bytes, Result},
};

// Specification per @bertani:
// [
//  uint8 versionByte,
//  bytes userData,
//  bytes4 origin protocol (bytes1) + origin chainid ( keccak256(whateverchainid)[:3] ),
//  origin sender
// ]

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Metadata {
    pub version: MetadataVersion,
    pub user_data: Bytes,
    pub chain_id: BlockchainChainId,
    pub origin_address: MetadataOriginAddress,
}

impl Metadata {
    pub fn new(user_data: &[Byte], origin_address: &MetadataOriginAddress) -> Self {
        Self::new_v1(user_data, origin_address)
    }

    pub fn new_v1(user_data: &[Byte], origin_address: &MetadataOriginAddress) -> Self {
        Self {
            version: MetadataVersion::V1,
            user_data: user_data.to_vec(),
            origin_address: origin_address.clone(),
            chain_id: origin_address.chain_id.clone(),
        }
    }

    fn to_bytes_for_eth(&self) -> Result<Bytes> {
        Ok(eth_abi_encode(&[
            EthAbiToken::FixedBytes(self.version.to_bytes()),
            EthAbiToken::Bytes(self.user_data.clone()),
            EthAbiToken::FixedBytes(self.chain_id.to_bytes()),
            EthAbiToken::Address(EthAddress::from_slice(&self.origin_address.to_bytes()?)),
        ]))
    }

    fn to_bytes(&self, destination_protocol: &BlockchainProtocolId) -> Result<Bytes> {
        match destination_protocol {
            BlockchainProtocolId::Ethereum => self.to_bytes_for_eth(),
            BlockchainProtocolId::Bitcoin | BlockchainProtocolId::Eos => {
                Err("Encoding metadata for Bitcoin || EOS is not implemented!".into())
            },
        }
    }

    fn get_err_msg(field: &str, protocol: &BlockchainProtocolId) -> String {
        format!(
            "Error getting `{}` from bytes for {} metadata!",
            field,
            protocol.to_symbol()
        )
    }

    fn from_bytes_from_eth(bytes: &[Byte]) -> Result<Self> {
        let protocol = BlockchainProtocolId::Ethereum;
        let tokens = eth_abi_decode(
            &[
                EthAbiParamType::FixedBytes(1),
                EthAbiParamType::Bytes,
                EthAbiParamType::FixedBytes(4),
                EthAbiParamType::Address,
            ],
            bytes,
        )?;
        let chain_id = match tokens[2] {
            EthAbiToken::FixedBytes(ref bytes) => BlockchainChainId::from_bytes(bytes),
            _ => Err(Self::get_err_msg("chain_id", &protocol).into()),
        }?;
        let eth_address = match tokens[3] {
            EthAbiToken::Address(address) => Ok(address),
            _ => Err(Self::get_err_msg("eth_address", &protocol)),
        }?;
        let version = match tokens[0] {
            EthAbiToken::FixedBytes(ref bytes) => MetadataVersion::from_bytes(bytes),
            _ => Err(Self::get_err_msg("version", &protocol).into()),
        }?;
        let user_data = match tokens[1] {
            EthAbiToken::Bytes(ref bytes) => Ok(bytes.clone()),
            _ => Err(Self::get_err_msg("user_data", &protocol)),
        }?;
        let origin_address = MetadataOriginAddress::new_from_eth_address(&eth_address, &chain_id)?;
        Ok(Self {
            version,
            user_data,
            origin_address,
            chain_id,
        })
    }

    pub fn from_bytes(bytes: &[Byte], protocol: &BlockchainProtocolId) -> Result<Self> {
        match protocol {
            BlockchainProtocolId::Ethereum => Self::from_bytes_from_eth(bytes),
            BlockchainProtocolId::Bitcoin | BlockchainProtocolId::Eos => {
                Err("Decoding metadata for Bitcoin || EOS is not implemented!".into())
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metadata::test_utils::get_sample_eth_metadata;

    #[test]
    fn should_make_eth_metadata_bytes_roundtrip() {
        let metadata = get_sample_eth_metadata();
        let bytes = metadata.to_bytes_for_eth().unwrap();
        let expected_bytes = hex::decode("01000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000080005fe7f9000000000000000000000000000000000000000000000000000000000000000000000000000000005a0b54d5dc17e0aadc383d2db43b0a0d3e029c4c0000000000000000000000000000000000000000000000000000000000000003c0ffee0000000000000000000000000000000000000000000000000000000000").unwrap();
        assert_eq!(bytes, expected_bytes);
        let result = Metadata::from_bytes_from_eth(&bytes).unwrap();
        assert_eq!(result, metadata);
    }
}
