use common::types::Result;
#[cfg(not(feature = "ltc"))]
use common_safe_addresses::{safely_convert_str_to_btc_address, SAFE_BTC_ADDRESS};
#[cfg(feature = "ltc")]
use common_safe_addresses::{safely_convert_str_to_ltc_address, SAFE_LTC_ADDRESS};
use derive_more::{Constructor, Deref, DerefMut};

use crate::{
    bitcoin_crate_alias::{blockdata::transaction::TxOut as BtcTxOut, Address as BtcAddress},
    btc_utils::create_new_tx_output,
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

#[cfg(feature = "ltc")]
impl Default for BtcRecipientAndAmount {
    fn default() -> Self {
        Self {
            amount: 0,
            recipient: SAFE_LTC_ADDRESS.clone(),
        }
    }
}

#[cfg(feature = "ltc")]
impl BtcRecipientAndAmount {
    pub fn new(recipient: &str, amount: u64) -> Result<Self> {
        Ok(BtcRecipientAndAmount {
            amount,
            recipient: safely_convert_str_to_ltc_address(recipient),
        })
    }
}

#[cfg(not(feature = "ltc"))]
impl Default for BtcRecipientAndAmount {
    fn default() -> Self {
        Self {
            amount: 0,
            recipient: SAFE_BTC_ADDRESS.clone(),
        }
    }
}

#[cfg(not(feature = "ltc"))]
impl BtcRecipientAndAmount {
    pub fn new(recipient: &str, amount: u64) -> Result<Self> {
        Ok(BtcRecipientAndAmount {
            amount,
            recipient: safely_convert_str_to_btc_address(recipient),
        })
    }
}
