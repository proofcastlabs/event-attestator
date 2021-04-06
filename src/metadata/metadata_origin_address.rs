use std::str::{from_utf8, FromStr};

use bitcoin::util::address::Address as BtcAddress;
use eos_chain::AccountName as EosAddress;
use ethereum_types::Address as EthAddress;

use crate::{
    chains::eth::eth_constants::ETH_ADDRESS_SIZE_IN_BYTES,
    metadata::{metadata_chain_id::MetadataChainId, metadata_protocol_id::MetadataProtocolId},
    types::{Byte, Bytes, Result},
    utils::strip_hex_prefix,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MetadataOriginAddress {
    pub address: String,
    pub metadata_chain_id: MetadataChainId,
}

impl MetadataOriginAddress {
    fn get_err_msg(protocol: MetadataProtocolId) -> String {
        let symbol = protocol.to_symbol();
        format!(
            "`MetadataOriginAddress` error - {} address supplied with non-{} chain ID!",
            symbol, symbol
        )
    }

    pub fn new_from_eth_address(eth_address: &EthAddress, metadata_chain_id: &MetadataChainId) -> Result<Self> {
        let protocol_id = metadata_chain_id.to_protocol_id();
        match protocol_id {
            MetadataProtocolId::Ethereum => Ok(Self {
                metadata_chain_id: *metadata_chain_id,
                address: hex::encode(eth_address),
            }),
            _ => Err(Self::get_err_msg(protocol_id).into()),
        }
    }

    pub fn new_from_eos_address(eos_address: &EosAddress, metadata_chain_id: &MetadataChainId) -> Result<Self> {
        let protocol_id = metadata_chain_id.to_protocol_id();
        match protocol_id {
            MetadataProtocolId::Eos => Ok(Self {
                metadata_chain_id: *metadata_chain_id,
                address: eos_address.to_string(),
            }),
            _ => Err(Self::get_err_msg(protocol_id).into()),
        }
    }

    pub fn new_from_btc_address(btc_address: &BtcAddress, metadata_chain_id: &MetadataChainId) -> Result<Self> {
        let protocol_id = metadata_chain_id.to_protocol_id();
        match protocol_id {
            MetadataProtocolId::Bitcoin => Ok(Self {
                metadata_chain_id: *metadata_chain_id,
                address: btc_address.to_string(),
            }),
            _ => Err(Self::get_err_msg(protocol_id).into()),
        }
    }

    pub fn to_bytes(&self) -> Result<Bytes> {
        match self.metadata_chain_id.to_protocol_id() {
            MetadataProtocolId::Bitcoin | MetadataProtocolId::Eos => Ok(self.address.as_bytes().to_vec()),
            MetadataProtocolId::Ethereum => Ok(hex::decode(strip_hex_prefix(&self.address))?),
        }
    }

    #[allow(dead_code)]
    fn from_bytes(bytes: &[Byte], metadata_chain_id: &MetadataChainId) -> Result<Self> {
        let protocol_id = metadata_chain_id.to_protocol_id();
        match protocol_id {
            MetadataProtocolId::Bitcoin => {
                info!("✔ Attempting to create `MetadataOriginAddress` from bytes for EOS...");
                match from_utf8(bytes) {
                    Err(err) => {
                        Err(format!("Error converting bytes to utf8 in `MetadataOriginAddress`: {}", err).into())
                    },
                    Ok(btc_address_str) => match BtcAddress::from_str(btc_address_str) {
                        Ok(ref btc_address) => Self::new_from_btc_address(btc_address, metadata_chain_id),
                        Err(err) => Err(format!(
                            "Error converting bytes to BTC address in `MetadataOriginAddress`: {}",
                            err
                        )
                        .into()),
                    },
                }
            },
            MetadataProtocolId::Eos => {
                info!("✔ Attempting to create `MetadataOriginAddress` from bytes for EOS...");
                match from_utf8(bytes) {
                    Err(err) => {
                        Err(format!("Error converting bytes to utf8 in `MetadataOriginAddress`: {}", err).into())
                    },
                    Ok(eos_address_str) => match EosAddress::from_str(eos_address_str) {
                        Ok(ref eos_address) => Self::new_from_eos_address(eos_address, metadata_chain_id),
                        Err(err) => Err(format!(
                            "Error converting bytes to EOS address in `MetadataOriginAddress`: {}",
                            err
                        )
                        .into()),
                    },
                }
            },
            MetadataProtocolId::Ethereum => {
                info!("✔ Attempting to create `MetadataOriginAddress` from bytes for ETH...");
                if bytes.len() == ETH_ADDRESS_SIZE_IN_BYTES {
                    Self::new_from_eth_address(&EthAddress::from_slice(bytes), metadata_chain_id)
                } else {
                    Err("Incorrect number of bytes to convert to ETH address in `MetadataOriginAddress`!".into())
                }
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metadata::test_utils::{
        get_sample_btc_address,
        get_sample_btc_origin_address,
        get_sample_eos_address,
        get_sample_eos_origin_address,
        get_sample_eth_address,
        get_sample_eth_origin_address,
    };

    #[test]
    fn should_get_metadata_origin_address_from_eos_address() {
        let metadata_chain_id = MetadataChainId::TelosMainnet;
        let result = MetadataOriginAddress::new_from_eos_address(&get_sample_eos_address(), &metadata_chain_id);
        assert!(result.is_ok());
    }

    #[test]
    fn should_get_metadata_origin_address_from_btc_address() {
        let metadata_chain_id = MetadataChainId::BitcoinMainnet;
        let result = MetadataOriginAddress::new_from_btc_address(&get_sample_btc_address(), &metadata_chain_id);
        assert!(result.is_ok());
    }

    #[test]
    fn should_get_metadata_origin_address_from_eth_address() {
        let metadata_chain_id = MetadataChainId::EthereumRopsten;
        let result = MetadataOriginAddress::new_from_eth_address(&get_sample_eth_address(), &metadata_chain_id);
        assert!(result.is_ok());
    }

    #[test]
    fn should_do_btc_address_bytes_roundtrip() {
        let metadata_origin_address = get_sample_btc_origin_address();
        let metadata_chain_id = metadata_origin_address.metadata_chain_id.clone();
        let bytes = metadata_origin_address.to_bytes().unwrap();
        let result = MetadataOriginAddress::from_bytes(&bytes, &metadata_chain_id).unwrap();
        assert_eq!(result, metadata_origin_address);
    }

    #[test]
    fn should_do_eth_address_bytes_roundtrip() {
        let metadata_origin_address = get_sample_eth_origin_address();
        let metadata_chain_id = metadata_origin_address.metadata_chain_id.clone();
        let bytes = metadata_origin_address.to_bytes().unwrap();
        let result = MetadataOriginAddress::from_bytes(&bytes, &metadata_chain_id).unwrap();
        assert_eq!(result, metadata_origin_address);
    }

    #[test]
    fn should_do_eos_address_bytes_roundtrip() {
        let metadata_origin_address = get_sample_eos_origin_address();
        let metadata_chain_id = metadata_origin_address.metadata_chain_id.clone();
        let bytes = metadata_origin_address.to_bytes().unwrap();
        let result = MetadataOriginAddress::from_bytes(&bytes, &metadata_chain_id).unwrap();
        assert_eq!(result, metadata_origin_address);
    }
}
