use std::fmt;

use common::{
    chains::eth::eth_utils::{convert_eth_address_to_string, convert_eth_hash_to_string},
    types::{Byte, Bytes, Result},
};
use derive_more::{Constructor, Deref, IntoIterator};
use ethereum_types::{Address as EthAddress, H256 as EthHash, U256};
use serde::{Deserialize, Serialize};
use serde_json;

#[derive(Debug, Clone, Default, PartialEq, Eq, Constructor, Deref, IntoIterator, Serialize, Deserialize)]
pub struct BtcOnIntBtcTxInfos(pub Vec<BtcOnIntBtcTxInfo>);

impl BtcOnIntBtcTxInfos {
    pub fn to_bytes(&self) -> Result<Bytes> {
        Ok(serde_json::to_vec(self)?)
    }

    pub fn from_bytes(bytes: &[Byte]) -> Result<Self> {
        if bytes.is_empty() {
            Ok(Self(vec![]))
        } else {
            Ok(serde_json::from_slice(bytes)?)
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Constructor, Serialize, Deserialize)]
pub struct BtcOnIntBtcTxInfo {
    pub to: EthAddress,
    pub from: EthAddress,
    pub recipient: String,
    pub amount_in_wei: U256,
    pub amount_in_satoshis: u64,
    pub token_address: EthAddress,
    pub originating_tx_hash: EthHash,
}

impl fmt::Display for BtcOnIntBtcTxInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "
BtcOnIntBtcTxInfo: {{
    to: {},
    from: {},
    recipient: {},
    amount_in_wei: {},
    amount_in_satoshis: {},
    token_address: {},
    originating_tx_has: {}
}}
",
            convert_eth_address_to_string(&self.to),
            convert_eth_address_to_string(&self.from),
            self.recipient,
            self.amount_in_wei,
            self.amount_in_satoshis,
            convert_eth_address_to_string(&self.token_address),
            convert_eth_hash_to_string(&self.originating_tx_hash),
        )
    }
}
