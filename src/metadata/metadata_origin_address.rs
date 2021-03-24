use std::str::{from_utf8, FromStr};

use bitcoin::util::address::Address as BtcAddress;
use eos_chain::AccountName as EosAddress;
use ethereum_types::Address as EthAddress;

use crate::{
    chains::eth::eth_constants::ETH_ADDRESS_SIZE_IN_BYTES,
    metadata::{blockchain_chain_id::BlockchainChainId, blockchain_protocol_id::BlockchainProtocolId},
    types::{Byte, Bytes, Result},
    utils::strip_hex_prefix,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MetadataOriginAddress {
    pub address: String,
    pub chain_id: BlockchainChainId,
}

impl MetadataOriginAddress {
    fn get_err_msg(protocol: BlockchainProtocolId) -> String {
        let symbol = protocol.to_symbol();
        format!(
            "`MetadataOriginAddress` error - {} address supplied with non-{} chain ID!",
            symbol, symbol
        )
    }

    pub fn new_from_eth_address(eth_address: &EthAddress, chain_id: &BlockchainChainId) -> Result<Self> {
        let protocol_id = chain_id.to_protocol_id();
        match protocol_id {
            BlockchainProtocolId::Ethereum => Ok(Self {
                chain_id: chain_id.clone(),
                address: hex::encode(eth_address),
            }),
            _ => Err(Self::get_err_msg(protocol_id).into()),
        }
    }

    pub fn new_from_eos_address(eos_address: &EosAddress, chain_id: &BlockchainChainId) -> Result<Self> {
        let protocol_id = chain_id.to_protocol_id();
        match protocol_id {
            BlockchainProtocolId::Eos => Ok(Self {
                chain_id: chain_id.clone(),
                address: eos_address.to_string(),
            }),
            _ => Err(Self::get_err_msg(protocol_id).into()),
        }
    }

    pub fn new_from_btc_address(btc_address: &BtcAddress, chain_id: &BlockchainChainId) -> Result<Self> {
        let protocol_id = chain_id.to_protocol_id();
        match protocol_id {
            BlockchainProtocolId::Bitcoin => Ok(Self {
                chain_id: chain_id.clone(),
                address: btc_address.to_string(),
            }),
            _ => Err(Self::get_err_msg(protocol_id).into()),
        }
    }

    pub fn to_bytes(&self) -> Result<Bytes> {
        match self.chain_id.to_protocol_id() {
            BlockchainProtocolId::Bitcoin | BlockchainProtocolId::Eos => Ok(self.address.as_bytes().to_vec()),
            BlockchainProtocolId::Ethereum => Ok(hex::decode(strip_hex_prefix(&self.address))?),
        }
    }

    pub fn from_bytes(bytes: &[Byte], chain_id: &BlockchainChainId) -> Result<Self> {
        let protocol_id = chain_id.to_protocol_id();
        match protocol_id {
            BlockchainProtocolId::Bitcoin => {
                info!("✔ Attempting to create `MetadataOriginAddress` from bytes for EOS...");
                match from_utf8(bytes) {
                    Err(err) => {
                        Err(format!("Error converting bytes to utf8 in `MetadataOriginAddress`: {}", err).into())
                    },
                    Ok(btc_address_str) => match BtcAddress::from_str(btc_address_str) {
                        Ok(ref btc_address) => Self::new_from_btc_address(btc_address, chain_id),
                        Err(err) => Err(format!(
                            "Error converting bytes to BTC address in `MetadataOriginAddress`: {}",
                            err
                        )
                        .into()),
                    },
                }
            },
            BlockchainProtocolId::Eos => {
                info!("✔ Attempting to create `MetadataOriginAddress` from bytes for EOS...");
                match from_utf8(bytes) {
                    Err(err) => {
                        Err(format!("Error converting bytes to utf8 in `MetadataOriginAddress`: {}", err).into())
                    },
                    Ok(eos_address_str) => match EosAddress::from_str(eos_address_str) {
                        Ok(ref eos_address) => Self::new_from_eos_address(eos_address, chain_id),
                        Err(err) => Err(format!(
                            "Error converting bytes to EOS address in `MetadataOriginAddress`: {}",
                            err
                        )
                        .into()),
                    },
                }
            },
            BlockchainProtocolId::Ethereum => {
                info!("✔ Attempting to create `MetadataOriginAddress` from bytes for ETH...");
                if bytes.len() == ETH_ADDRESS_SIZE_IN_BYTES {
                    Self::new_from_eth_address(&EthAddress::from_slice(bytes), chain_id)
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
        let chain_id = BlockchainChainId::TelosMainnet;
        let result = MetadataOriginAddress::new_from_eos_address(&get_sample_eos_address(), &chain_id);
        assert!(result.is_ok());
    }

    #[test]
    fn should_get_metadata_origin_address_from_btc_address() {
        let chain_id = BlockchainChainId::BitcoinMainnet;
        let result = MetadataOriginAddress::new_from_btc_address(&get_sample_btc_address(), &chain_id);
        assert!(result.is_ok());
    }

    #[test]
    fn should_get_metadata_origin_address_from_eth_address() {
        let chain_id = BlockchainChainId::EthereumRopsten;
        let result = MetadataOriginAddress::new_from_eth_address(&get_sample_eth_address(), &chain_id);
        assert!(result.is_ok());
    }

    #[test]
    fn should_do_btc_address_bytes_roundtrip() {
        let metadata_origin_address = get_sample_btc_origin_address();
        let chain_id = metadata_origin_address.chain_id.clone();
        let bytes = metadata_origin_address.to_bytes().unwrap();
        let result = MetadataOriginAddress::from_bytes(&bytes, &chain_id).unwrap();
        assert_eq!(result, metadata_origin_address);
    }

    #[test]
    fn should_do_eth_address_bytes_roundtrip() {
        let metadata_origin_address = get_sample_eth_origin_address();
        let chain_id = metadata_origin_address.chain_id.clone();
        let bytes = metadata_origin_address.to_bytes().unwrap();
        let result = MetadataOriginAddress::from_bytes(&bytes, &chain_id).unwrap();
        assert_eq!(result, metadata_origin_address);
    }

    #[test]
    fn should_do_eos_address_bytes_roundtrip() {
        let metadata_origin_address = get_sample_eos_origin_address();
        let chain_id = metadata_origin_address.chain_id.clone();
        let bytes = metadata_origin_address.to_bytes().unwrap();
        let result = MetadataOriginAddress::from_bytes(&bytes, &chain_id).unwrap();
        assert_eq!(result, metadata_origin_address);
    }

    // #[test]
    // fn should_fail_to_perform_bytes_round_trip_for_wrong_protocol_id() {
    // let metadata_origin_address = MetadataOriginAddress::new_from_btc_address(&get_sample_btc_address());
    // let wrong_protocol_id = BlockchainProtocolId::Eos;
    // assert_ne!(wrong_protocol_id, BlockchainProtocolId::Bitcoin);
    // let bytes = metadata_origin_address.to_bytes().unwrap();
    // let result = MetadataOriginAddress::from_bytes(&bytes, &wrong_protocol_id);
    // assert!(result.is_err());
    // }
}
