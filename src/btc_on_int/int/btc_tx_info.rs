use std::fmt;

use derive_more::{Constructor, Deref, IntoIterator};
use ethereum_types::{Address as EthAddress, H256 as EthHash, U256};

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
            "BtcOnIntBtcTxInfo: {{
                to: {},
                from: {},
                recipient: {},
                amount_in_wei: {},
                amount_in_satoshis: {},
                token_address: {},
                originating_tx_has: {}
            }}",
            self.to,
            self.from,
            self.recipient,
            self.amount_in_wei,
            self.amount_in_satoshis,
            self.token_address,
            self.originating_tx_hash,
        )
    }
}
