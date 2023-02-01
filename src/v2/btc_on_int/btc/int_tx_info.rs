use bitcoin::Txid;
use derive_more::{Constructor, Deref, DerefMut};
use ethereum_types::{Address as EthAddress, U256};
use serde::{Deserialize, Serialize};

use crate::{
    address::Address,
    chains::btc::btc_constants::ZERO_HASH,
    safe_addresses::SAFE_ETH_ADDRESS_STR,
    types::{Byte, Bytes, Result},
};

#[derive(Default, Debug, Clone, PartialEq, Eq, Deref, DerefMut, Constructor, Serialize, Deserialize)]
pub struct BtcOnIntIntTxInfos(pub Vec<BtcOnIntIntTxInfo>);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Constructor)]
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
    pub vault_address: EthAddress,
}

impl Default for BtcOnIntIntTxInfo {
    fn default() -> Self {
        Self {
            // NOTE: The `rust_bitcoin` lib removed default from Txid. Didn't bump the major though so :/
            originating_tx_hash: Txid::from(*ZERO_HASH),
            // NOTE: And we can't use `..Default::default()` for the rest without recursing, sigh.
            host_token_amount: U256::default(),
            user_data: Bytes::default(),
            int_token_address: EthAddress::default(),
            originating_tx_address: String::default(),
            destination_address: String::default(),
            origin_chain_id: Bytes::default(),
            destination_chain_id: Bytes::default(),
            router_address: EthAddress::default(),
            native_token_amount: u64::default(),
            vault_address: EthAddress::default(),
        }
    }
}

impl BtcOnIntIntTxInfos {
    pub fn to_bytes(&self) -> Result<Bytes> {
        Ok(serde_json::to_vec(&self.0)?)
    }

    pub fn from_bytes(bytes: &[Byte]) -> Result<Self> {
        Ok(serde_json::from_slice(bytes)?)
    }
}

impl_tx_info_trait!(
    BtcOnIntIntTxInfo,
    vault_address,
    router_address,
    int_token_address,
    destination_address,
    Address::Eth,
    SAFE_ETH_ADDRESS_STR
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_should_work_for_btc_on_int_int_tx_info() {
        BtcOnIntIntTxInfo::default();
    }
}
