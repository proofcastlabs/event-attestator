use std::fmt;

use derive_more::{Constructor, Deref, IntoIterator};
use ethereum_types::{Address as EthAddress, H256 as EthHash, U256};

use crate::chains::eth::eth_utils::{convert_eth_address_to_string, convert_eth_hash_to_string};

#[derive(Debug, Clone, Default, PartialEq, Eq, Constructor, Deref, IntoIterator)]
pub struct BtcOnIntBtcTxInfos(pub Vec<BtcOnIntBtcTxInfo>);

#[derive(Debug, Default, Clone, PartialEq, Eq, Constructor)]
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
