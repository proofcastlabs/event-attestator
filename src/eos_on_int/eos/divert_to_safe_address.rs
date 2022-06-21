use crate::{
    chains::eos::eos_state::EosState,
    eos_on_int::eos::int_tx_info::{EosOnIntIntTxInfo, EosOnIntIntTxInfos},
    safe_addresses::SAFE_ETH_ADDRESS_HEX,
    traits::DatabaseInterface,
    types::Result,
};

create_safe_address_diversion_fxns!(
    "EosOnIntIntTxInfo" => EosState => "int" => SAFE_ETH_ADDRESS_HEX.to_string() => String => "token"
);
