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
