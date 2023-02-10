use common::{safe_addresses::SAFE_EOS_ADDRESS_STR, traits::DatabaseInterface, types::Result};
use common_eth::EthState;

use crate::int::eos_tx_info::{IntOnEosEosTxInfo, IntOnEosEosTxInfos};

create_safe_address_diversion_fxns_v2!(
    "IntOnEosEosTxInfo" => EthState => "eos" => SAFE_EOS_ADDRESS_STR.to_string() => String => "token"
);
