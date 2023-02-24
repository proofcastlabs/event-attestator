use std::str::FromStr;

use common::{
    address::Address,
    safe_addresses::SAFE_ETH_ADDRESS_STR,
    types::{Byte, Bytes, Result},
};
use common_metadata::MetadataChainId;
use derive_more::{Constructor, Deref, IntoIterator};
use ethereum_types::{Address as EthAddress, U256};
use rust_algorand::AlgorandAddress;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct IntOnAlgoIntTxInfo {
    pub user_data: Bytes,
    pub algo_asset_id: u64,
    pub native_token_amount: U256,
    pub router_address: EthAddress,
    pub destination_address: String,
    pub originating_tx_hash: String,
    pub int_vault_address: EthAddress,
    pub token_sender: AlgorandAddress,
    pub int_token_address: EthAddress,
    pub origin_chain_id: MetadataChainId,
    pub destination_chain_id: MetadataChainId,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Constructor, Deref, IntoIterator, Serialize, Deserialize)]
struct IntOnAlgoIntTxInfosSerdable(pub Vec<IntOnAlgoIntTxInfoSerdable>);

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct IntOnAlgoIntTxInfoSerdable {
    user_data: Bytes,
    algo_asset_id: u64,
    native_token_amount: U256,
    router_address: EthAddress,
    destination_address: String,
    originating_tx_hash: String,
    int_vault_address: EthAddress,
    token_sender: String,
    int_token_address: EthAddress,
    origin_chain_id: MetadataChainId,
    destination_chain_id: MetadataChainId,
}

impl IntOnAlgoIntTxInfosSerdable {
    fn from_tx_infos(tx_infos: &IntOnAlgoIntTxInfos) -> Self {
        Self::new(
            tx_infos
                .iter()
                .map(IntOnAlgoIntTxInfoSerdable::from_tx_info)
                .collect::<Vec<_>>(),
        )
    }

    fn to_tx_infos(&self) -> Result<IntOnAlgoIntTxInfos> {
        Ok(IntOnAlgoIntTxInfos::new(
            self.iter()
                .map(IntOnAlgoIntTxInfoSerdable::to_tx_info)
                .collect::<Result<Vec<_>>>()?,
        ))
    }
}

impl IntOnAlgoIntTxInfos {
    pub fn from_bytes(bytes: &[Byte]) -> Result<Self> {
        if bytes.is_empty() {
            Ok(Self::default())
        } else {
            serde_json::from_slice::<IntOnAlgoIntTxInfosSerdable>(bytes)?.to_tx_infos()
        }
    }

    pub fn to_bytes(&self) -> Result<Bytes> {
        Ok(serde_json::to_vec(&IntOnAlgoIntTxInfosSerdable::from_tx_infos(self))?)
    }
}

impl IntOnAlgoIntTxInfoSerdable {
    fn to_tx_info(&self) -> Result<IntOnAlgoIntTxInfo> {
        Ok(IntOnAlgoIntTxInfo {
            user_data: self.user_data.clone(),
            algo_asset_id: self.algo_asset_id,
            router_address: self.router_address,
            origin_chain_id: self.origin_chain_id,
            int_vault_address: self.int_vault_address,
            int_token_address: self.int_token_address,
            native_token_amount: self.native_token_amount,
            destination_address: self.destination_address.clone(),
            originating_tx_hash: self.originating_tx_hash.clone(),
            destination_chain_id: self.destination_chain_id,
            token_sender: AlgorandAddress::from_str(&self.token_sender)?,
        })
    }

    fn from_tx_info(tx_info: &IntOnAlgoIntTxInfo) -> Self {
        Self {
            user_data: tx_info.user_data.clone(),
            algo_asset_id: tx_info.algo_asset_id,
            router_address: tx_info.router_address,
            token_sender: tx_info.token_sender.to_string(),
            origin_chain_id: tx_info.origin_chain_id,
            int_vault_address: tx_info.int_vault_address,
            int_token_address: tx_info.int_token_address,
            native_token_amount: tx_info.native_token_amount,
            destination_address: tx_info.destination_address.clone(),
            originating_tx_hash: tx_info.originating_tx_hash.clone(),
            destination_chain_id: tx_info.destination_chain_id,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Constructor, Deref, IntoIterator)]
pub struct IntOnAlgoIntTxInfos(pub Vec<IntOnAlgoIntTxInfo>);

impl_tx_info_trait!(
    IntOnAlgoIntTxInfo,
    int_vault_address,
    router_address,
    int_token_address,
    destination_address,
    Address::Eth,
    SAFE_ETH_ADDRESS_STR
);
