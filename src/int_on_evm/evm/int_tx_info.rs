use derive_more::{Constructor, Deref, IntoIterator};
use ethereum_types::{Address as EthAddress, H256 as EthHash, U256};

use crate::{metadata::metadata_chain_id::MetadataChainId, types::Bytes};

#[derive(Debug, Default, Clone, PartialEq, Eq, Constructor)]
pub struct IntOnEvmIntTxInfo {
    pub user_data: Bytes,
    pub token_sender: EthAddress,
    pub native_token_amount: U256,
    pub router_address: EthAddress,
    pub destination_address: String,
    pub originating_tx_hash: EthHash,
    pub evm_token_address: String,
    pub eth_token_address: String,
    pub origin_chain_id: MetadataChainId,
    pub destination_chain_id: MetadataChainId,
    pub eth_vault_address: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Constructor, Deref, IntoIterator)]
pub struct IntOnEvmIntTxInfos(pub Vec<IntOnEvmIntTxInfo>);
