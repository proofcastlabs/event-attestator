pub use bitcoin::{
    blockdata::{
        block::{Block as BtcBlock, BlockHeader as BtcBlockHeader},
        transaction::{Transaction as BtcTransaction, TxOut as BtcTxOut},
    },
    consensus::encode::deserialize as btc_deserialize,
    hashes::sha256d,
    util::address::Address as BtcAddress,
};
use serde::{Deserialize, Serialize};

use crate::{
    chains::btc::{btc_constants::BTC_PUB_KEY_SLICE_LENGTH, deposit_address_info::DepositAddressInfoJson},
    types::{Byte, Bytes},
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
