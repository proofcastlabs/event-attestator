use std::fmt;

use derive_more::{Constructor, Deref, IntoIterator};
use ethereum_types::{Address as EthAddress, H256 as EthHash, U256};

use crate::{
    chains::eth::eth_utils::{convert_eth_address_to_string, convert_eth_hash_to_string},
    metadata::metadata_chain_id::MetadataChainId,
    types::Bytes,
    utils::convert_bytes_to_string,
};

#[derive(Debug, Default, Clone, PartialEq, Eq, Constructor)]
pub struct IntOnEvmIntTxInfo {
    pub user_data: Bytes,
    pub token_sender: EthAddress,
    pub native_token_amount: U256,
    pub router_address: EthAddress,
    pub destination_address: String,
    pub originating_tx_hash: EthHash,
    pub evm_token_address: EthAddress,
    pub eth_token_address: String,
    pub origin_chain_id: MetadataChainId,
    pub destination_chain_id: MetadataChainId,
    pub vault_address: EthAddress,
}

#[derive(Debug, Clone, PartialEq, Eq, Constructor, Deref, IntoIterator)]
pub struct IntOnEvmIntTxInfos(pub Vec<IntOnEvmIntTxInfo>);

impl fmt::Display for IntOnEvmIntTxInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "
IntOnEvmIntTxInfo: {{
    token_sender: {},
    native_token_amount: {},
    router_address: {},
    destination_address: {},
    originating_tx_hash: {},
    evm_token_address: {},
    eth_token_address: {},
    origin_chain_id: {},
    destination_chain_id: {},
    vault_address: {},
    user_data: {},
}}
            ",
            convert_eth_address_to_string(&self.token_sender),
            self.native_token_amount,
            convert_eth_address_to_string(&self.router_address),
            self.destination_address,
            convert_eth_hash_to_string(&self.originating_tx_hash),
            convert_eth_address_to_string(&self.evm_token_address),
            self.eth_token_address,
            self.origin_chain_id,
            self.destination_chain_id,
            convert_eth_address_to_string(&self.vault_address),
            convert_bytes_to_string(&self.user_data),
        )
    }
}
