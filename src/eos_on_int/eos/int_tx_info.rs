use derive_more::{Constructor, Deref};
use eos_chain::{AccountName as EosAccountName, Checksum256};
use ethereum_types::{Address as EthAddress, U256};
use serde::{Deserialize, Serialize};

use crate::{
    address::Address,
    chains::eos::eos_global_sequences::GlobalSequence,
    metadata::MetadataChainId,
    safe_addresses::SAFE_ETH_ADDRESS_STR,
    types::Bytes,
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Deref, Constructor)]
pub struct EosOnIntIntTxInfos(pub Vec<EosOnIntIntTxInfo>);

#[derive(Clone, Debug, Default, PartialEq, Eq, Constructor, Serialize, Deserialize)]
pub struct EosOnIntIntTxInfo {
    pub amount: U256,
    pub user_data: Bytes,
    pub eos_tx_amount: String,
    pub eos_token_address: String,
    pub vault_address: EthAddress,
    pub router_address: EthAddress,
    pub destination_address: String,
    pub int_token_address: EthAddress,
    pub origin_address: EosAccountName,
    pub originating_tx_id: Checksum256,
    pub global_sequence: GlobalSequence,
    pub origin_chain_id: MetadataChainId,
    pub destination_chain_id: MetadataChainId,
}

impl_tx_info_trait!(
    EosOnIntIntTxInfo,
    vault_address,
    router_address,
    int_token_address,
    destination_address,
    Address::Eth,
    SAFE_ETH_ADDRESS_STR
);
