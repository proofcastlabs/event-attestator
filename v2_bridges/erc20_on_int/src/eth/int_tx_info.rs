use std::fmt;

use common::{
    address::Address,
    safe_addresses::SAFE_ETH_ADDRESS_STR,
    types::{Byte, Bytes, Result},
    utils::convert_bytes_to_string,
};
use common_eth::{convert_eth_address_to_string, convert_eth_hash_to_string};
use common_metadata::MetadataChainId;
use derive_more::{Constructor, Deref};
use ethereum_types::{Address as EthAddress, H256 as EthHash, U256};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Erc20OnIntIntTxInfo {
    pub user_data: Bytes,
    pub token_sender: EthAddress,
    pub vault_address: EthAddress,
    pub native_token_amount: U256,
    pub router_address: EthAddress,
    pub destination_address: String,
    pub originating_tx_hash: EthHash,
    pub evm_token_address: EthAddress,
    pub eth_token_address: EthAddress,
    pub origin_chain_id: MetadataChainId,
    pub destination_chain_id: MetadataChainId,
}

#[derive(Debug, Clone, PartialEq, Eq, Constructor, Serialize, Deserialize, Deref)]
pub struct Erc20OnIntIntTxInfos(pub Vec<Erc20OnIntIntTxInfo>);

impl Erc20OnIntIntTxInfos {
    pub fn to_bytes(&self) -> Result<Bytes> {
        Ok(serde_json::to_vec(self)?)
    }

    pub fn from_bytes(bytes: &[Byte]) -> Result<Self> {
        Ok(serde_json::from_slice(bytes)?)
    }
}

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

impl_tx_info_trait!(
    Erc20OnIntIntTxInfo,
    vault_address,
    router_address,
    evm_token_address,
    destination_address,
    Address::Eth,
    SAFE_ETH_ADDRESS_STR
);
