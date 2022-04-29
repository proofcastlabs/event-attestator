use crate::{
    chains::eos::eos_state::EosState,
    int_on_eos::eos::int_tx_info::{IntOnEosIntTxInfo, IntOnEosIntTxInfos},
    safe_addresses::SAFE_ETH_ADDRESS_HEX,
    traits::DatabaseInterface,
    types::Result,
};

create_safe_address_diversion_fxns!(
    "IntOnEosIntTxInfo" => EosState => "int" => SAFE_ETH_ADDRESS_HEX.to_string() => String => "token", "vault"
);
