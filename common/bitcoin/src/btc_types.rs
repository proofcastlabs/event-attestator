use bitcoin::{blockdata::transaction::Transaction as BtcTransaction, hashes::sha256d};
use common::types::{Byte, Bytes};
use serde::{Deserialize, Serialize};

use crate::{btc_constants::BTC_PUB_KEY_SLICE_LENGTH, deposit_address_info::DepositAddressInfoJson};

#[derive(
    Debug,
    Default,
    Clone,
    Eq,
    PartialEq,
    derive_more::Constructor,
    derive_more::Deref,
    serde::Serialize,
    serde::Deserialize,
)]
pub struct BtcTransactions(Vec<BtcTransaction>);

impl common::traits::Serdable for BtcTransactions {}

pub type BtcPubKeySlice = [Byte; BTC_PUB_KEY_SLICE_LENGTH];

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct BtcUtxoAndValue {
    pub value: u64,
    pub serialized_utxo: Bytes,
    pub maybe_extra_data: Option<Bytes>,
    pub maybe_pointer: Option<sha256d::Hash>,
    pub maybe_deposit_info_json: Option<DepositAddressInfoJson>,
}
