use ethereum_types::{
    U256,
    H256 as EthHash,
    Address as EthAddress,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RedeemInfo {
    pub amount: U256,
    pub from: EthAddress,
    pub recipient: String,
    pub originating_tx_hash: EthHash,
}

impl RedeemInfo {
    pub fn new(amount: U256, from: EthAddress, recipient: String, originating_tx_hash: EthHash) -> RedeemInfo {
        RedeemInfo { amount, recipient, originating_tx_hash, from }
    }
}
