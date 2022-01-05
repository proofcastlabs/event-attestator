#![cfg(test)]
use std::str::FromStr;

use bitcoin::util::address::Address as BtcAddress;
use eos_chain::AccountName as EosAddress;
use ethereum_types::Address as EthAddress;

use crate::{
    metadata::{
        metadata_address::MetadataAddress,
        metadata_chain_id::MetadataChainId,
        metadata_version::MetadataVersion,
        Metadata,
    },
    types::Bytes,
};

pub fn get_sample_eos_address() -> EosAddress {
    EosAddress::from_str("aneosaddress").unwrap()
}

pub fn get_sample_btc_address() -> BtcAddress {
    BtcAddress::from_str("12dRugNcdxK39288NjcDV4GX7rMsKCGn6B").unwrap()
}

pub fn get_sample_eth_address() -> EthAddress {
    EthAddress::from_slice(&hex::decode("5A0b54D5dc17e0AadC383d2db43B0a0D3E029c4c").unwrap())
}

fn get_sample_user_data() -> Bytes {
    vec![0xc0, 0xff, 0xee]
}

pub fn get_sample_eth_origin_address() -> MetadataAddress {
    MetadataAddress::new_from_eth_address(&get_sample_eth_address(), &MetadataChainId::EthereumMainnet).unwrap()
}

pub fn get_sample_eos_origin_address() -> MetadataAddress {
    MetadataAddress::new_from_eos_address(&get_sample_eos_address(), &MetadataChainId::EosMainnet).unwrap()
}

pub fn get_sample_btc_origin_address() -> MetadataAddress {
    MetadataAddress::new_from_btc_address(&get_sample_btc_address(), &MetadataChainId::BitcoinMainnet).unwrap()
}

pub fn get_sample_eth_metadata() -> Metadata {
    Metadata::new(&get_sample_user_data(), &get_sample_eth_origin_address())
}

pub fn get_sample_eth_metadata_v2() -> Metadata {
    Metadata {
        version: MetadataVersion::V2,
        user_data: hex::decode("d3caff").unwrap(),
        origin_chain_id: MetadataChainId::EthereumRinkeby,
        origin_address: MetadataAddress {
            address: "fEDFe2616EB3661CB8FEd2782F5F0cC91D59DCaC".to_string(),
            metadata_chain_id: MetadataChainId::EthereumRinkeby,
        },
        destination_chain_id: Some(MetadataChainId::EthereumRopsten),
        destination_address: Some(MetadataAddress {
            address: "edB86cd455ef3ca43f0e227e00469C3bDFA40628".to_string(),
            metadata_chain_id: MetadataChainId::EthereumRopsten,
        }),
        protocol_options: None,
        protocol_receipt: None,
    }
}

pub fn get_sample_eos_metadata() -> Metadata {
    Metadata::new(&get_sample_user_data(), &get_sample_eos_origin_address())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_get_sample_eth_metadata() {
        get_sample_eth_metadata();
    }

    #[test]
    fn should_get_sample_eos_metadata() {
        get_sample_eos_metadata();
    }
}
