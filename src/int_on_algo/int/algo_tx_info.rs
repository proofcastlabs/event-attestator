use derive_more::{Constructor, Deref};
use ethereum_types::{Address as EthAddress, H256 as EthHash, U256};
use rust_algorand::{AlgorandAddress, AlgorandAppId};

use crate::{
    metadata::metadata_chain_id::MetadataChainId,
    types::{Bytes, Result},
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
