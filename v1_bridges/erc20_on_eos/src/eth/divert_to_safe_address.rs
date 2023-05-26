use common::{traits::DatabaseInterface, types::Result};
use common_eth::EthState;
use common_safe_addresses::SAFE_EOS_ADDRESS_STR;

use crate::eth::eos_tx_info::{Erc20OnEosEosTxInfo, Erc20OnEosEosTxInfos};

create_safe_address_diversion_fxns_v2!(
    "Erc20OnEosEosTxInfo" => EthState => "eos" => SAFE_EOS_ADDRESS_STR.to_string() => String => "token"
);
