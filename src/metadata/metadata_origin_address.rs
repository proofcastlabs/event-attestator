use std::str::{from_utf8, FromStr};

use bitcoin::util::address::Address as BtcAddress;
use eos_chain::AccountName as EosAddress;
use ethereum_types::Address as EthAddress;

use crate::{
    chains::eth::eth_constants::ETH_ADDRESS_SIZE_IN_BYTES,
    metadata::blockchain_protocol_id::BlockchainProtocolId,
    types::{Byte, Bytes, Result},
    utils::strip_hex_prefix,
};

#[derive(Clone, Debug, PartialEq, Eq)]
struct MetadataOriginAddress {
    pub address: String,
    pub protocol_id: BlockchainProtocolId,
}

impl MetadataOriginAddress {
    pub fn new_from_eth_address(eth_address: &EthAddress) -> Self {
        Self {
            address: hex::encode(eth_address),
            protocol_id: BlockchainProtocolId::Ethereum,
        }
    }

    pub fn new_from_eos_address(eos_address: &EosAddress) -> Self {
        Self {
            address: eos_address.to_string(),
            protocol_id: BlockchainProtocolId::Eos,
        }
    }

    pub fn new_from_btc_address(btc_address: &BtcAddress) -> Self {
        Self {
            address: btc_address.to_string(),
            protocol_id: BlockchainProtocolId::Bitcoin,
        }
    }

    pub fn to_bytes(&self) -> Result<Bytes> {
        match self.protocol_id {
            BlockchainProtocolId::Bitcoin | BlockchainProtocolId::Eos => Ok(self.address.as_bytes().to_vec()),
            BlockchainProtocolId::Ethereum => Ok(hex::decode(strip_hex_prefix(&self.address))?),
        }
    }

    pub fn from_bytes(bytes: &[Byte], protocol_id: &BlockchainProtocolId) -> Result<Self> {
        match protocol_id {
            BlockchainProtocolId::Bitcoin => {
                info!("✔ Attempting to create `MetadataOriginAddress` from bytes for EOS...");
                match from_utf8(bytes) {
                    Err(err) => {
                        Err(format!("Error converting bytes to utf8 in `MetadataOriginAddress`: {}", err).into())
                    },
                    Ok(btc_address_str) => match BtcAddress::from_str(btc_address_str) {
                        Ok(ref btc_address) => Ok(Self::new_from_btc_address(btc_address)),
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
                        Ok(ref eos_address) => Ok(Self::new_from_eos_address(eos_address)),
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
                    Ok(Self::new_from_eth_address(&EthAddress::from_slice(bytes)))
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

    fn get_sample_eos_address() -> EosAddress {
        EosAddress::from_str("aneosaddress").unwrap()
    }

    fn get_sample_btc_address() -> BtcAddress {
        BtcAddress::from_str("12dRugNcdxK39288NjcDV4GX7rMsKCGn6B").unwrap()
    }

    fn get_sample_eth_address() -> EthAddress {
        EthAddress::from_slice(&hex::decode("5A0b54D5dc17e0AadC383d2db43B0a0D3E029c4c").unwrap())
    }

    #[test]
    fn should_get_metadata_origin_address_from_eos_address() {
        let result = MetadataOriginAddress::new_from_eos_address(&get_sample_eos_address());
        assert!(result.protocol_id == BlockchainProtocolId::Eos)
    }

    #[test]
    fn should_get_metadata_origin_address_from_btc_address() {
        let result = MetadataOriginAddress::new_from_btc_address(&get_sample_btc_address());
        assert!(result.protocol_id == BlockchainProtocolId::Bitcoin)
    }

    #[test]
    fn should_get_metadata_origin_address_from_eth_address() {
        let result = MetadataOriginAddress::new_from_eth_address(&get_sample_eth_address());
        assert!(result.protocol_id == BlockchainProtocolId::Ethereum)
    }

    #[test]
    fn should_do_btc_address_bytes_roundtrip() {
        let metadata_origin_address = MetadataOriginAddress::new_from_btc_address(&get_sample_btc_address());
        let protocol_id = BlockchainProtocolId::Bitcoin;
        let bytes = metadata_origin_address.to_bytes().unwrap();
        let result = MetadataOriginAddress::from_bytes(&bytes, &protocol_id).unwrap();
        assert_eq!(result, metadata_origin_address);
    }

    #[test]
    fn should_do_eth_address_bytes_roundtrip() {
        let metadata_origin_address = MetadataOriginAddress::new_from_eth_address(&get_sample_eth_address());
        let protocol_id = BlockchainProtocolId::Ethereum;
        let bytes = metadata_origin_address.to_bytes().unwrap();
        let result = MetadataOriginAddress::from_bytes(&bytes, &protocol_id).unwrap();
        assert_eq!(result, metadata_origin_address);
    }

    #[test]
    fn should_do_eos_address_bytes_roundtrip() {
        let metadata_origin_address = MetadataOriginAddress::new_from_eos_address(&get_sample_eos_address());
        let protocol_id = BlockchainProtocolId::Eos;
        let bytes = metadata_origin_address.to_bytes().unwrap();
        let result = MetadataOriginAddress::from_bytes(&bytes, &protocol_id).unwrap();
        assert_eq!(result, metadata_origin_address);
    }

    #[test]
    fn should_fail_to_perform_bytes_round_trip_for_wrong_protocol_id() {
        let metadata_origin_address = MetadataOriginAddress::new_from_btc_address(&get_sample_btc_address());
        let wrong_protocol_id = BlockchainProtocolId::Eos;
        assert_ne!(wrong_protocol_id, BlockchainProtocolId::Bitcoin);
        let bytes = metadata_origin_address.to_bytes().unwrap();
        let result = MetadataOriginAddress::from_bytes(&bytes, &wrong_protocol_id);
        assert!(result.is_err());
    }
}
