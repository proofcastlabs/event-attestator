use derive_more::{Constructor, Deref, IntoIterator};
use ethereum_types::{Address as EthAddress, U256};
use rust_algorand::AlgorandAddress;

use crate::{
    address::Address,
    metadata::metadata_chain_id::MetadataChainId,
    safe_addresses::SAFE_ETH_ADDRESS_STR,
    types::Bytes,
};

#[derive(Debug, Default, Clone, PartialEq, Eq, Constructor)]
pub struct IntOnAlgoIntTxInfo {
    pub user_data: Bytes,
    pub algo_asset_id: u64,
    pub native_token_amount: U256,
    pub router_address: EthAddress,
    pub destination_address: String,
    pub originating_tx_hash: String,
    pub int_vault_address: EthAddress,
    pub token_sender: AlgorandAddress,
    pub int_token_address: EthAddress,
    pub origin_chain_id: MetadataChainId,
    pub destination_chain_id: MetadataChainId,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Constructor, Deref, IntoIterator)]
pub struct IntOnAlgoIntTxInfos(pub Vec<IntOnAlgoIntTxInfo>);

impl_tx_info_trait!(
    IntOnAlgoIntTxInfo,
    int_vault_address,
    router_address,
    int_token_address,
    destination_address,
    Address::Eth,
    SAFE_ETH_ADDRESS_STR
);
