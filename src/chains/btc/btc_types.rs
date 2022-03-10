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
use serde::{Deserialize, Serialize};

use crate::{
    chains::btc::{
        btc_constants::BTC_PUB_KEY_SLICE_LENGTH,
        btc_utils::create_new_tx_output,
        deposit_address_info::DepositAddressInfoJson,
    },
    safe_addresses::{SAFE_BTC_ADDRESS, SAFE_BTC_ADDRESS_STR},
    types::{Byte, Bytes, Result},
};

pub type BtcTransactions = Vec<BtcTransaction>;
pub type BtcPubKeySlice = [Byte; BTC_PUB_KEY_SLICE_LENGTH];

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct BtcUtxoAndValue {
    pub value: u64,
    pub serialized_utxo: Bytes,
    pub maybe_extra_data: Option<Bytes>,
    pub maybe_pointer: Option<sha256d::Hash>,
    pub maybe_deposit_info_json: Option<DepositAddressInfoJson>,
}

// FIXME TODO Move to own mod!
#[derive(Debug, Clone, Eq, PartialEq, Default, Deref, DerefMut, Constructor)]
pub struct BtcRecipientsAndAmounts(Vec<BtcRecipientAndAmount>);

impl BtcRecipientsAndAmounts {
    pub fn to_tx_outputs(&self) -> Vec<BtcTxOut> {
        self.iter()
            .flat_map(|recipient_and_amount| {
                create_new_tx_output(
                    recipient_and_amount.amount,
                    recipient_and_amount.recipient.script_pubkey(),
                )
            })
            .collect()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BtcRecipientAndAmount {
    pub amount: u64,
    pub recipient: BtcAddress,
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
