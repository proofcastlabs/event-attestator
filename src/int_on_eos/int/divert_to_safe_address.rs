use crate::{
    chains::eth::eth_state::EthState,
    int_on_eos::int::eos_tx_info::{IntOnEosEosTxInfo, IntOnEosEosTxInfos},
    safe_addresses::SAFE_EOS_ADDRESS_STR,
    traits::DatabaseInterface,
    types::Result,
};

create_safe_address_diversion_fxns!(
    "IntOnEosEosTxInfo" => EthState => "eos" => SAFE_EOS_ADDRESS_STR.to_string() => String => "token"
);
