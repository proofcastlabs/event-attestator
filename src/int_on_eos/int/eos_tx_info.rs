use std::fmt;

use derive_more::{Constructor, Deref};
use ethereum_types::{Address as EthAddress, H256 as EthHash, U256};

use crate::{
    chains::eth::eth_utils::{convert_eth_address_to_string, convert_eth_hash_to_string},
    metadata::metadata_chain_id::MetadataChainId,
    types::Bytes,
    utils::convert_bytes_to_string,
};

#[derive(Debug, Clone, PartialEq, Eq, Constructor, Deref)]
pub struct IntOnEosEosTxInfos(pub Vec<IntOnEosEosTxInfo>);

#[derive(Debug, Clone, PartialEq, Eq, Default, Constructor)]
pub struct IntOnEosEosTxInfo {
    pub user_data: Bytes,
    pub token_amount: U256,
    pub router_address: String,
    pub eos_asset_amount: String,
    pub token_sender: EthAddress,
    pub vault_address: EthAddress,
    pub eos_token_address: String,
    pub destination_address: String,
    pub originating_tx_hash: EthHash,
    pub eth_token_address: EthAddress,
    pub origin_chain_id: MetadataChainId,
    pub destination_chain_id: MetadataChainId,
}

impl fmt::Display for IntOnEosEosTxInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "
IntOnEosEosTxInfo: {{
    token_amount: {},
    vault_address: {},
    router_address: {},
    eos_asset_amount: {},
    token_sender: {},
    eos_token_address: {},
    destination_address: {},
    originating_tx_hash: {},
    eth_token_address: {},
    origin_chain_id: {},
    destination_chain_id: {},
    user_data: {},
}}
            ",
            self.token_amount,
            convert_eth_address_to_string(&self.vault_address),
            self.router_address,
            self.eos_asset_amount,
            convert_eth_address_to_string(&self.token_sender),
            self.eos_token_address,
            self.destination_address,
            convert_eth_hash_to_string(&self.originating_tx_hash),
            convert_eth_address_to_string(&self.eth_token_address),
            self.origin_chain_id,
            self.destination_chain_id,
            convert_bytes_to_string(&self.user_data),
        )
    }
}
