use bitcoin::{util::address::Address as BtcAddress, Txid};
use derive_more::{Constructor, Deref, DerefMut};
use ethereum_types::{Address as EthAddress, U256};
use serde::{Deserialize, Serialize};

use crate::{
    chains::{btc::btc_metadata::ToMetadata, eth::eth_utils::safely_convert_hex_to_eth_address},
    metadata::metadata_chain_id::MetadataChainId,
    types::{Byte, Bytes, Result},
};

#[derive(Debug, Default, Clone, PartialEq, Eq, Deref, DerefMut, Constructor, Serialize, Deserialize)]
pub struct BtcOnIntIntTxInfos(pub Vec<BtcOnIntIntTxInfo>);

#[derive(Clone, Default, Debug, PartialEq, Eq, Serialize, Deserialize, Constructor)]
pub struct BtcOnIntIntTxInfo {
    pub host_token_amount: U256,
    pub user_data: Bytes,
    pub originating_tx_hash: Txid,
    pub int_token_address: EthAddress,
    pub originating_tx_address: String,
    pub destination_address: String,
    pub origin_chain_id: Bytes,
    pub destination_chain_id: Bytes,
    pub router_address: EthAddress,
    pub native_token_amount: u64,
}

impl BtcOnIntIntTxInfos {
    pub fn to_bytes(&self) -> Result<Bytes> {
        Ok(serde_json::to_vec(&self.0)?)
    }

    pub fn from_bytes(bytes: &[Byte]) -> Result<Self> {
        Ok(serde_json::from_slice(bytes)?)
    }
}

impl ToMetadata for BtcOnIntIntTxInfo {
    // TODO V2 metadata!!
    fn get_user_data(&self) -> Option<Bytes> {
        if self.user_data.is_empty() {
            None
        } else {
            Some(self.user_data.clone())
        }
    }

    fn get_originating_tx_address(&self) -> String {
        self.originating_tx_address.clone()
    }
}

// TODO test!
