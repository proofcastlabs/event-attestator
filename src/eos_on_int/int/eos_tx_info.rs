use derive_more::{Constructor, Deref};
use ethereum_types::{Address as EthAddress, H256 as EthHash, U256};

use crate::{metadata::MetadataChainId, types::Bytes};

#[derive(Debug, Clone, PartialEq, Eq, Constructor, Deref)]
pub struct EosOnIntEosTxInfos(pub Vec<EosOnIntEosTxInfo>);

#[derive(Debug, Clone, PartialEq, Eq, Default, Constructor)]
pub struct EosOnIntEosTxInfo {
    pub user_data: Bytes,
    pub token_amount: U256,
    pub router_address: String,
    pub eos_asset_amount: String,
    pub token_sender: EthAddress,
    pub eos_token_address: String,
    pub destination_address: String,
    pub originating_tx_hash: EthHash,
    pub int_token_address: EthAddress,
    pub origin_chain_id: MetadataChainId,
    pub destination_chain_id: MetadataChainId,
}
