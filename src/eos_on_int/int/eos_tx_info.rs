use derive_more::{Constructor, Deref};
use ethereum_types::{Address as EthAddress, H256 as EthHash, U256};

use crate::{chains::eth::eth_chain_id::EthChainId, types::Bytes};

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct EosOnIntEosTxInfo {
    pub user_data: Bytes,
    pub token_amount: U256,
    pub token_sender: EthAddress,
    pub eos_asset_amount: String,
    pub eos_token_address: String,
    pub origin_chain_id: EthChainId, // FIXME
    pub destination_address: String,
    pub originating_tx_hash: EthHash,
    pub int_token_address: EthAddress,
}

#[derive(Debug, Clone, PartialEq, Eq, Constructor, Deref)]
pub struct EosOnIntEosTxInfos(pub Vec<EosOnIntEosTxInfo>);
