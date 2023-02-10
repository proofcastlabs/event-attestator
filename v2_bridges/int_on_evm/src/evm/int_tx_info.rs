use std::fmt;

use common::{
    address::Address,
    metadata::metadata_chain_id::MetadataChainId,
    safe_addresses::SAFE_ETH_ADDRESS_STR,
    types::{Byte, Bytes, Result},
    utils::convert_bytes_to_string,
};
use common_eth::{convert_eth_address_to_string, convert_eth_hash_to_string};
use derive_more::{Constructor, Deref, IntoIterator};
use ethereum_types::{Address as EthAddress, H256 as EthHash, U256};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IntOnEvmIntTxInfo {
    pub user_data: Bytes,
    pub host_token_amount: U256,
    pub token_sender: EthAddress,
    pub native_token_amount: U256,
    pub vault_address: EthAddress,
    pub router_address: EthAddress,
    pub destination_address: String,
    pub originating_tx_hash: EthHash,
    pub evm_token_address: EthAddress,
    pub eth_token_address: EthAddress,
    pub origin_chain_id: MetadataChainId,
    pub destination_chain_id: MetadataChainId,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Constructor, Deref, IntoIterator, Serialize, Deserialize)]
pub struct IntOnEvmIntTxInfos(pub Vec<IntOnEvmIntTxInfo>);

impl IntOnEvmIntTxInfos {
    pub fn from_bytes(bytes: &[Byte]) -> Result<Self> {
        if bytes.is_empty() {
            Ok(Self::default())
        } else {
            Ok(serde_json::from_slice(bytes)?)
        }
    }

    pub fn to_bytes(&self) -> Result<Bytes> {
        Ok(serde_json::to_vec(self)?)
    }
}

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
    host_token_amount: {},
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
            self.host_token_amount,
            convert_eth_hash_to_string(&self.originating_tx_hash),
            convert_eth_address_to_string(&self.evm_token_address),
            convert_eth_address_to_string(&self.eth_token_address),
            self.origin_chain_id,
            self.destination_chain_id,
            convert_eth_address_to_string(&self.vault_address),
            convert_bytes_to_string(&self.user_data),
        )
    }
}

impl_tx_info_trait!(
    IntOnEvmIntTxInfo,
    vault_address,
    router_address,
    eth_token_address,
    destination_address,
    Address::Eth,
    SAFE_ETH_ADDRESS_STR
);
