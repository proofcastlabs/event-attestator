use crate::{
    types::Bytes,
    chains::btc::deposit_address_info::{
        DepositInfoList,
        DepositAddressInfoJson,
    },
};
use bitcoin::{
    hashes::sha256d,
    blockdata::block::Block as BtcBlock,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BtcBlockAndId {
    pub height: u64,
    pub block: BtcBlock,
    pub id: sha256d::Hash,
    pub deposit_address_list: DepositInfoList,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct BtcUtxoAndValue {
    pub value: u64,
    pub serialized_utxo: Bytes,
    pub maybe_extra_data: Option<Bytes>,
    pub maybe_pointer: Option<sha256d::Hash>,
    pub maybe_deposit_info_json: Option<DepositAddressInfoJson>,
}
