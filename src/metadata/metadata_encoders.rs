use ethabi::{encode as eth_abi_encode, Token as EthAbiToken};
use ethereum_types::Address as EthAddress;

use crate::{
    chains::eos::eos_metadata::EosMetadata,
    metadata::{metadata_protocol_id::MetadataProtocolId, metadata_version::MetadataVersion, Metadata},
    types::{Bytes, Result},
};

impl Metadata {
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

    // NOTE: Unlike v2 encoding, v3 encodes the addresses as `string` types in the EVM. This allows
    // us to be generic w/r/t host and native chain's address types that surround the interim chain.
    fn to_bytes_for_eth_v3(&self) -> Result<Bytes> {
        Ok(eth_abi_encode(&[
            EthAbiToken::FixedBytes(self.version.to_bytes()),
            EthAbiToken::Bytes(self.user_data.clone()),
            EthAbiToken::FixedBytes(self.origin_chain_id.to_bytes()?),
            match self.origin_address.metadata_chain_id.to_protocol_id() {
                MetadataProtocolId::Ethereum => EthAbiToken::String(self.origin_address.to_string()),
                MetadataProtocolId::Eos | MetadataProtocolId::Bitcoin => {
                    EthAbiToken::Bytes(self.origin_address.to_bytes()?)
                },
            },
            EthAbiToken::FixedBytes(self.get_destination_chain_id()?.to_bytes()?),
            EthAbiToken::String(match &self.destination_address {
                Some(metadata_address) => Result::Ok(metadata_address.to_string()),
                None => Err("No `destination_address` in metadata!".into()),
            }?),
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
            MetadataVersion::V3 => self.to_bytes_for_eth_v3(),
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
    use crate::metadata::test_utils::{
        get_sample_eth_metadata,
        get_sample_eth_metadata_v2,
        get_sample_eth_metadata_v3,
    };

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

    #[test]
    fn should_encode_v3_metadata_for_eth() {
        let metadata = get_sample_eth_metadata_v3();
        let result = hex::encode(metadata.to_bytes_for_eth().unwrap());
        let expected_result = "0300000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000010000f343680000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001400069c3220000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001a0000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000002200000000000000000000000000000000000000000000000000000000000000003d3caff0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002a30786645444665323631364542333636314342384645643237383246354630634339314435394443614300000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002a3078656442383663643435356566336361343366306532323765303034363943336244464134303632380000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000";
        assert_eq!(result, expected_result);
    }
}
