use crate::{
    chains::eth::eth_state::EthState,
    eos_on_int::int::eos_tx_info::{EosOnIntEosTxInfo, EosOnIntEosTxInfos},
    safe_addresses::SAFE_EOS_ADDRESS_STR,
    traits::DatabaseInterface,
    types::Result,
};

create_safe_address_diversion_fxns!(
    "EosOnIntEosTxInfo" => EthState => "eos" => SAFE_EOS_ADDRESS_STR.to_string() => String =>"token"
);
