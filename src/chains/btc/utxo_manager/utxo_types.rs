use crate::{
    types::{
        Bytes,
        Result,
    },
    chains::btc::{
        deposit_address_info::DepositAddressInfoJson,
        btc_utils::{
            serialize_btc_utxo,
            deserialize_btc_utxo,
        },
    },
};
use bitcoin::{
    hashes::sha256d,
    blockdata::transaction::TxIn as BtcUtxo,
};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct BtcUtxosAndValues(pub Vec<BtcUtxoAndValue>);

impl BtcUtxosAndValues {
    pub fn new(utxos_and_values: Vec<BtcUtxoAndValue>) -> Self {
        Self(utxos_and_values)
    }

    pub fn new_empty() -> Self {
        Self(Vec::new())
    }

    pub fn push(mut self, utxo_and_value: BtcUtxoAndValue) -> Self {
        self.0.push(utxo_and_value);
        self
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn append(mut self, mut utxos_and_values: BtcUtxosAndValues) -> Self {
        self.0.append(&mut utxos_and_values.0);
        self
    }

    pub fn sum(&self) -> u64 {
        self.0.iter().map(|utxo_and_value| utxo_and_value.value).sum()
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct BtcUtxoAndValue {
    pub value: u64,
    pub serialized_utxo: Bytes,
    pub maybe_extra_data: Option<Bytes>,
    pub maybe_pointer: Option<sha256d::Hash>,
    pub maybe_deposit_info_json: Option<DepositAddressInfoJson>,
}

impl BtcUtxoAndValue {
    pub fn new(
        value: u64,
        utxo: &BtcUtxo,
        maybe_deposit_info_json: Option<DepositAddressInfoJson>,
        maybe_extra_data: Option<Bytes>,
    ) -> Self {
        BtcUtxoAndValue {
            value,
            maybe_extra_data,
            maybe_pointer: None,
            maybe_deposit_info_json,
            serialized_utxo: serialize_btc_utxo(utxo),
        }
    }

    pub fn get_utxo(&self) -> Result<BtcUtxo> {
        deserialize_btc_utxo(&self.serialized_utxo)
    }

    pub fn update_pointer(mut self, hash: sha256d::Hash) -> Self {
        self.maybe_pointer = Some(hash);
        self
    }
}
