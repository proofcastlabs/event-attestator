use common::{chains::btc::BtcState, safe_addresses::SAFE_EOS_ADDRESS_STR, traits::DatabaseInterface, types::Result};

use crate::btc::eos_tx_info::{BtcOnEosEosTxInfo, BtcOnEosEosTxInfos};

create_safe_address_diversion_fxns_v2!(
    "BtcOnEosEosTxInfo" => BtcState => "eos" => SAFE_EOS_ADDRESS_STR.to_string() => String => "token"
);
