use std::fmt;

use derive_more::{Constructor, Deref};
use ethereum_types::{Address as EthAddress, H256 as EthHash, U256};

use crate::{
    chains::eth::eth_utils::{convert_eth_address_to_string, convert_eth_hash_to_string},
    metadata::metadata_chain_id::MetadataChainId,
    types::Bytes,
    utils::convert_bytes_to_string,
};

#[derive(Debug, Clone, Default, PartialEq, Eq, Constructor)]
pub struct Erc20OnIntIntTxInfo {
    pub user_data: Bytes,
    pub token_sender: EthAddress,
    pub vault_address: EthAddress,
    pub native_token_amount: U256,
    pub evm_token_address: String,
    pub router_address: EthAddress,
    pub destination_address: String,
    pub originating_tx_hash: EthHash,
    pub eth_token_address: EthAddress,
    pub origin_chain_id: MetadataChainId,
    pub destination_chain_id: MetadataChainId,
}

#[derive(Debug, Clone, PartialEq, Eq, Constructor, Deref)]
pub struct Erc20OnIntIntTxInfos(pub Vec<Erc20OnIntIntTxInfo>);

impl fmt::Display for Erc20OnIntIntTxInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "
Erc20OnIntIntTxInfo: {{
    token_sender: {},
    native_token_amount: {},
    vault_address: {},
    evm_token_address: {},
    router_address: {},
    destination_address: {},
    originating_tx_hash: {},
    eth_token_address: {},
    origin_chain_id: {},
    destination_chain_id: {},
    user_data: {},
}}
",
            convert_eth_address_to_string(&self.token_sender),
            self.native_token_amount,
            convert_eth_address_to_string(&self.vault_address),
            self.evm_token_address,
            convert_eth_address_to_string(&self.router_address),
            self.destination_address,
            convert_eth_hash_to_string(&self.originating_tx_hash),
            convert_eth_address_to_string(&self.eth_token_address),
            self.origin_chain_id,
            self.destination_chain_id,
            convert_bytes_to_string(&self.user_data),
        )
    }
}
