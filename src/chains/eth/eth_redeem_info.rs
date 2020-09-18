use ethereum_types::{
    U256,
    H256 as EthHash,
    Address as EthAddress,
};
use crate::{
    types::Result,
    btc_on_eth::btc::btc_types::{
        BtcRecipientAndAmount,
        BtcRecipientsAndAmounts,
    },
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RedeemInfos(pub Vec<RedeemInfo>);

impl RedeemInfos {
    pub fn new(redeem_infos: Vec<RedeemInfo>) -> Self {
        Self(redeem_infos)
    }

    pub fn sum(&self) -> u64 {
        self.0.iter().map(|params| params.amount.as_u64()).sum()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn to_btc_addresses_and_amounts(&self) -> Result<BtcRecipientsAndAmounts> {
        info!("✔ Getting BTC addresses & amounts from redeem params...");
        self
            .0
            .iter()
            .map(|params| {
                let recipient_and_amount = BtcRecipientAndAmount::new(&params.recipient[..], params.amount.as_u64());
                info!("✔ Recipients & amount retrieved from redeem: {:?}", recipient_and_amount);
                recipient_and_amount
             })
            .collect()
    }
}
