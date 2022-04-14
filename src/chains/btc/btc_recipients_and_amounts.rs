use std::str::FromStr;

pub use bitcoin::{
    blockdata::{
        block::{Block as BtcBlock, BlockHeader as BtcBlockHeader},
        transaction::{Transaction as BtcTransaction, TxOut as BtcTxOut},
    },
    consensus::encode::deserialize as btc_deserialize,
    hashes::sha256d,
    util::address::Address as BtcAddress,
};
use derive_more::{Constructor, Deref, DerefMut};

use crate::{
    chains::btc::btc_utils::create_new_tx_output,
    safe_addresses::{SAFE_BTC_ADDRESS, SAFE_BTC_ADDRESS_STR},
    types::Result,
};

#[derive(Debug, Clone, Eq, PartialEq, Default, Deref, DerefMut, Constructor)]
pub struct BtcRecipientsAndAmounts(Vec<BtcRecipientAndAmount>);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BtcRecipientAndAmount {
    pub amount: u64,
    pub recipient: BtcAddress,
}

impl BtcRecipientsAndAmounts {
    pub fn to_tx_outputs(&self) -> Vec<BtcTxOut> {
        self.iter()
            .map(|recipient_and_amount| {
                create_new_tx_output(
                    recipient_and_amount.amount,
                    recipient_and_amount.recipient.script_pubkey(),
                )
            })
            .collect()
    }

    pub fn sum(&self) -> u64 {
        self.iter()
            .map(|recipient_and_amount| recipient_and_amount.amount)
            .sum()
    }
}

impl Default for BtcRecipientAndAmount {
    fn default() -> Self {
        Self {
            amount: 0,
            recipient: SAFE_BTC_ADDRESS.clone(),
        }
    }
}

impl BtcRecipientAndAmount {
    pub fn new(recipient: &str, amount: u64) -> Result<Self> {
        Ok(BtcRecipientAndAmount {
            amount,
            recipient: match BtcAddress::from_str(recipient) {
                Ok(address) => address,
                Err(error) => {
                    info!("✔ Error parsing BTC address for recipient: {}", error);
                    info!("✔ Defaulting to SAFE BTC address: {}", SAFE_BTC_ADDRESS_STR);
                    BtcAddress::from_str(SAFE_BTC_ADDRESS_STR)?
                },
            },
        })
    }
}
