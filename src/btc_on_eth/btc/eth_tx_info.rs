use bitcoin::{util::address::Address as BtcAddress, Txid};
use derive_more::{Constructor, Deref, DerefMut};
use ethereum_types::{Address as EthAddress, U256};
use serde::{Deserialize, Serialize};

use crate::{
    chains::btc::{btc_constants::ZERO_HASH, btc_metadata::ToMetadata},
    safe_addresses::safely_convert_str_to_eth_address,
    types::{Byte, Bytes, Result},
};

#[derive(Debug, Clone, Default, PartialEq, Eq, Deref, DerefMut, Constructor, Serialize, Deserialize)]
pub struct BtcOnEthEthTxInfos(pub Vec<BtcOnEthEthTxInfo>);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BtcOnEthEthTxInfo {
    pub amount: U256,
    pub user_data: Option<Bytes>,
    pub originating_tx_hash: Txid,
    pub eth_token_address: EthAddress,
    pub originating_tx_address: String,
    pub destination_address: EthAddress,
}

impl Default for BtcOnEthEthTxInfo {
    fn default() -> Self {
        Self {
            // NOTE: The `rust_bitcoin` lib removed default from Txid. Didn't bump the major though so :/
            originating_tx_hash: Txid::from(*ZERO_HASH),
            // NOTE: And we can't use `..Default::default()` for the rest without recursing :/
            amount: U256::default(),
            user_data: Option::default(),
            eth_token_address: EthAddress::default(),
            originating_tx_address: String::default(),
            destination_address: EthAddress::default(),
        }
    }
}

impl BtcOnEthEthTxInfos {
    pub fn to_bytes(&self) -> Result<Bytes> {
        Ok(serde_json::to_vec(&self.0)?)
    }

    pub fn from_bytes(bytes: &[Byte]) -> Result<Self> {
        Ok(serde_json::from_slice(bytes)?)
    }
}

impl BtcOnEthEthTxInfo {
    pub fn new(
        amount: U256,
        eth_address_hex: String,
        originating_tx_hash: Txid,
        originating_tx_address: BtcAddress,
        user_data: Option<Bytes>,
        eth_token_address: &EthAddress,
    ) -> Result<BtcOnEthEthTxInfo> {
        Ok(BtcOnEthEthTxInfo {
            amount,
            originating_tx_hash,
            originating_tx_address: originating_tx_address.to_string(),
            destination_address: safely_convert_str_to_eth_address(&eth_address_hex),
            user_data,
            eth_token_address: *eth_token_address,
        })
    }
}

impl ToMetadata for BtcOnEthEthTxInfo {
    fn get_user_data(&self) -> Option<Bytes> {
        self.user_data.clone()
    }

    fn get_originating_tx_address(&self) -> String {
        self.originating_tx_address.clone()
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use bitcoin::{hashes::Hash, util::address::Address as BtcAddress};
    use ethereum_types::H160 as EthAddress;

    use super::*;
    use crate::{
        btc_on_eth::{test_utils::get_sample_eth_tx_infos, utils::convert_satoshis_to_wei},
        chains::{btc::btc_chain_id::BtcChainId, eth::eth_constants::MAX_BYTES_FOR_ETH_USER_DATA},
        metadata::metadata_protocol_id::MetadataProtocolId,
    };

    #[test]
    fn should_serde_eth_tx_infos() {
        let expected_serialization = "5b7b22616d6f756e74223a2230786332386632313963343030222c22757365725f64617461223a6e756c6c2c226f726967696e6174696e675f74785f68617368223a2239653864643239663038333938643761646639323532386163313133626363373336663761646364376339396565653034363861393932633831663365613938222c226574685f746f6b656e5f61646472657373223a22307830303030303030303030303030303030303030303030303030303030303030303030303030303030222c226f726967696e6174696e675f74785f61646472657373223a22324e324c48596274384b314b44426f6764365855473956427635594d36786566644d32222c2264657374696e6174696f6e5f61646472657373223a22307866656466653236313665623336363163623866656432373832663566306363393164353964636163227d5d";
        let amount = convert_satoshis_to_wei(1337);
        let originating_tx_address = BtcAddress::from_str("2N2LHYbt8K1KDBogd6XUG9VBv5YM6xefdM2").unwrap();
        let destination_address =
            EthAddress::from_slice(&hex::decode("fedfe2616eb3661cb8fed2782f5f0cc91d59dcac").unwrap());
        let user_data = None;
        let eth_token_address = EthAddress::default();
        let originating_tx_hash =
            Txid::from_slice(&hex::decode("98eaf3812c998a46e0ee997ccdadf736c7bc13c18a5292df7a8d39089fd28d9e").unwrap())
                .unwrap();
        let eth_tx_info = BtcOnEthEthTxInfo::new(
            amount,
            hex::encode(destination_address),
            originating_tx_hash,
            originating_tx_address,
            user_data,
            &eth_token_address,
        )
        .unwrap();
        let eth_tx_infos = BtcOnEthEthTxInfos::new(vec![eth_tx_info]);
        let serialized_eth_tx_infos = eth_tx_infos.to_bytes().unwrap();
        assert_eq!(hex::encode(&serialized_eth_tx_infos), expected_serialization);
        let deserialized = BtcOnEthEthTxInfos::from_bytes(&serialized_eth_tx_infos).unwrap();
        assert_eq!(deserialized.len(), eth_tx_infos.len());
        deserialized
            .iter()
            .enumerate()
            .for_each(|(i, eth_tx_info)| assert_eq!(eth_tx_info, &eth_tx_infos[i]));
    }

    #[test]
    fn should_convert_eth_tx_infos_to_metadata_bytes() {
        let mut eth_tx_info = get_sample_eth_tx_infos()[0].clone();
        eth_tx_info.user_data = Some(hex::decode("d3caffc0ff33").unwrap());
        let expected_result = Some(hex::decode("0100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000008001ec97de0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000c00000000000000000000000000000000000000000000000000000000000000006d3caffc0ff330000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002231333643544552616f636d38644c6245747a436146744a4a58396a6646686e43684b000000000000000000000000000000000000000000000000000000000000").unwrap());
        let btc_chain_id = BtcChainId::Bitcoin;
        let destination_protocol_id = MetadataProtocolId::Ethereum;
        let result = eth_tx_info
            .maybe_to_metadata_bytes(&btc_chain_id, MAX_BYTES_FOR_ETH_USER_DATA, &destination_protocol_id)
            .unwrap();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_not_convert_eth_tx_infos_to_metadata_bytes_if_user_data_too_large() {
        let mut eth_tx_info = get_sample_eth_tx_infos()[0].clone();
        eth_tx_info.user_data = Some(vec![0u8; MAX_BYTES_FOR_ETH_USER_DATA + 1]);
        let btc_chain_id = BtcChainId::Bitcoin;
        let destination_protocol_id = MetadataProtocolId::Ethereum;
        let result = eth_tx_info
            .maybe_to_metadata_bytes(&btc_chain_id, MAX_BYTES_FOR_ETH_USER_DATA, &destination_protocol_id)
            .unwrap();
        assert!(result.is_none());
    }
}
