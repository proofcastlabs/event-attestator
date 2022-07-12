use std::fmt;

use derive_more::{Constructor, Deref};
use ethereum_types::{Address as EthAddress, H256 as EthHash, U256};
use rust_algorand::{AlgorandAddress, AlgorandAppId};

use crate::{
    chains::eth::eth_utils::{convert_eth_address_to_string, convert_eth_hash_to_string},
    metadata::metadata_chain_id::MetadataChainId,
    types::{Bytes, Result},
    utils::convert_bytes_to_string,
};

#[derive(Debug, Default, Clone, PartialEq, Eq, Constructor)]
pub struct IntOnAlgoAlgoTxInfo {
    pub user_data: Bytes,
    pub algo_asset_id: u64,
    pub token_sender: String,
    pub host_token_amount: U256,
    pub native_token_amount: U256,
    pub router_address: EthAddress,
    pub originating_tx_hash: EthHash,
    pub int_token_address: EthAddress,
    pub origin_chain_id: MetadataChainId,
    pub destination_chain_id: MetadataChainId,
    pub issuance_manager_app_id: AlgorandAppId,
    pub destination_app_id: Option<AlgorandAppId>,
    pub destination_address: Option<AlgorandAddress>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Constructor, Deref)]
pub struct IntOnAlgoAlgoTxInfos(pub Vec<IntOnAlgoAlgoTxInfo>);

impl IntOnAlgoAlgoTxInfo {
    pub fn destination_is_app(&self) -> bool {
        self.destination_app_id.is_some()
    }

    pub fn get_destination_address(&self) -> Result<AlgorandAddress> {
        match &self.destination_address {
            Some(address) => Ok(*address),
            None => Err("No `destination_address` in `IntOnAlgoAlgoTxInfo` - is it an app peg-in?".into()),
        }
    }

    pub fn get_destination_app_id(&self) -> Result<AlgorandAppId> {
        match &self.destination_app_id {
            Some(app_id) => Ok(app_id.clone()),
            None => Err("No `destination_app_id` in `IntOnAlgoAlgoTxInfo` - is it an address peg-in?".into()),
        }
    }
}

impl fmt::Display for IntOnAlgoAlgoTxInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "
IntOnAlgoAlgoTxInfo: {{
    algo_asset_id: {},
    token_sender: {},
    host_token_amount: {},
    native_token_amount: {},
    router_address: {},
    originating_tx_hash: {},
    int_token_address: {},
    origin_chain_id: {},
    destination_chain_id: {},
    issuance_manager_app_id: {},
    destination_app_id: {},
    destination_address: {},
    user_data: {},
}}
            ",
            self.algo_asset_id,
            self.token_sender,
            self.host_token_amount,
            self.native_token_amount,
            convert_eth_address_to_string(&self.router_address),
            convert_eth_hash_to_string(&self.originating_tx_hash),
            convert_eth_address_to_string(&self.int_token_address),
            self.origin_chain_id,
            self.destination_chain_id,
            self.issuance_manager_app_id,
            match self.destination_app_id.as_ref() {
                Some(address) => address.to_string(),
                None => "Cannot unwrap `destination_app_id` option!".to_string(),
            },
            match self.destination_address.as_ref() {
                Some(address) => address.to_string(),
                None => "Cannot unwrap `destination_address` option!".to_string(),
            },
            convert_bytes_to_string(&self.user_data),
        )
    }
}
