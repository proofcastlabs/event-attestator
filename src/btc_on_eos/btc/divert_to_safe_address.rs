use crate::{
    btc_on_eos::btc::eos_tx_info::{BtcOnEosEosTxInfo, BtcOnEosEosTxInfos},
    chains::btc::btc_state::BtcState,
    create_eos_safe_address_diversion_fxns,
    traits::DatabaseInterface,
    types::Result,
};

create_eos_safe_address_diversion_fxns!("BtcOnEosEosTxInfo" => "Btc" => "token");
