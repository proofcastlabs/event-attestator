use crate::{
    btc_on_eos::btc::eos_tx_info::{BtcOnEosEosTxInfo, BtcOnEosEosTxInfos},
    chains::btc::btc_state::BtcState,
    safe_addresses::SAFE_EOS_ADDRESS_STR,
    traits::DatabaseInterface,
    types::Result,
};

create_safe_address_diversion_fxns!(
    "BtcOnEosEosTxInfo" => BtcState => "eos" => SAFE_EOS_ADDRESS_STR.to_string() => String => "token"
);
