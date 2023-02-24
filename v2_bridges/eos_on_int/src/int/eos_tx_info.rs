use common::types::{Byte, Bytes, Result};
use common_metadata::MetadataChainId;
use derive_more::{Constructor, Deref};
use ethereum_types::{Address as EthAddress, H256 as EthHash, U256};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Constructor, Deref, Serialize, Deserialize)]
pub struct EosOnIntEosTxInfos(pub Vec<EosOnIntEosTxInfo>);

impl EosOnIntEosTxInfos {
    pub fn to_bytes(&self) -> Result<Bytes> {
        Ok(serde_json::to_vec(&self)?)
    }

    pub fn from_bytes(bytes: &[Byte]) -> Result<Self> {
        Ok(serde_json::from_slice(bytes)?)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct EosOnIntEosTxInfo {
    pub user_data: Bytes,
    pub token_amount: U256,
    pub router_address: String,
    pub eos_asset_amount: String,
    pub token_sender: EthAddress,
    pub eos_token_address: String,
    pub destination_address: String,
    pub originating_tx_hash: EthHash,
    pub int_token_address: EthAddress,
    pub origin_chain_id: MetadataChainId,
    pub destination_chain_id: MetadataChainId,
}
