#![cfg(test)]
use std::str::FromStr;

use bitcoin::util::address::Address as BtcAddress;
use eos_chain::AccountName as EosAddress;
use ethereum_types::Address as EthAddress;
use rust_algorand::AlgorandAddress;

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

pub fn get_sample_algo_address() -> AlgorandAddress {
    AlgorandAddress::from_str("HIBVFSZFK4FEANCOZFIVZNBHLJK3ERRHKDRZVGX4RZU7WQIMSSKL4PQZMA").unwrap()
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

pub fn get_sample_algo_origin_address() -> MetadataAddress {
    MetadataAddress::new(
        &get_sample_algo_address().to_string(),
        &MetadataChainId::AlgorandMainnet,
    )
    .unwrap()
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
            address: "0xfedfe2616eb3661cb8fed2782f5f0cc91d59dcac".to_string(),
            metadata_chain_id: MetadataChainId::EthereumRinkeby,
        },
        destination_chain_id: Some(MetadataChainId::EthereumRopsten),
        destination_address: Some(MetadataAddress {
            address: "0xedb86cd455ef3ca43f0e227e00469c3bdfa40628".to_string(),
            metadata_chain_id: MetadataChainId::EthereumRopsten,
        }),
        protocol_options: None,
        protocol_receipt: None,
    }
}

pub fn get_sample_eth_metadata_v3() -> Metadata {
    let mut mutable_metadata = get_sample_eth_metadata_v2();
    mutable_metadata.version = MetadataVersion::V3;
    mutable_metadata
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
