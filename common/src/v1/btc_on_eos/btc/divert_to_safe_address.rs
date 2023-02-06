use crate::{
    btc_on_eos::btc::eos_tx_info::{BtcOnEosEosTxInfo, BtcOnEosEosTxInfos},
    safe_addresses::SAFE_EOS_ADDRESS_STR,
    state::BtcState,
    traits::DatabaseInterface,
    types::Result,
};

create_safe_address_diversion_fxns_v2!(
    "BtcOnEosEosTxInfo" => BtcState => "eos" => SAFE_EOS_ADDRESS_STR.to_string() => String => "token"
);
