use derive_more::{Constructor, Deref, IntoIterator};
use ethereum_types::{Address as EthAddress, H256 as EthHash};

#[derive(Debug, Clone, Default, PartialEq, Eq, Constructor, Deref, IntoIterator)]
pub struct BtcOnIntBtcTxInfos(pub Vec<BtcOnIntBtcTxInfo>);

#[derive(Debug, Default, Clone, PartialEq, Eq, Constructor)]
pub struct BtcOnIntBtcTxInfo {
    pub amount_in_satoshis: u64,
    pub from: EthAddress,
    pub recipient: String,
    pub originating_tx_hash: EthHash,
}
