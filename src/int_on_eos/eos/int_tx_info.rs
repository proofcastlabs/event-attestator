use derive_more::{Constructor, Deref};
use eos_chain::{AccountName as EosAccountName, Checksum256};
use ethereum_types::{Address as EthAddress, U256};

use crate::{
    chains::eos::eos_global_sequences::GlobalSequence,
    metadata::metadata_chain_id::MetadataChainId,
    types::Bytes,
};

#[derive(Clone, Debug, PartialEq, Eq, Deref, Constructor)]
pub struct IntOnEosIntTxInfos(pub Vec<IntOnEosIntTxInfo>);

#[derive(Clone, Debug, Default, PartialEq, Eq, Constructor)]
pub struct IntOnEosIntTxInfo {
    pub amount: U256,
    pub user_data: Bytes,
    pub eos_tx_amount: String,
    pub eos_token_address: String,
    pub int_token_address: String,
    pub int_vault_address: String,
    pub router_address: EthAddress,
    pub destination_address: String,
    pub origin_address: EosAccountName,
    pub originating_tx_id: Checksum256,
    pub global_sequence: GlobalSequence,
    pub origin_chain_id: MetadataChainId,
    pub destination_chain_id: MetadataChainId,
}
