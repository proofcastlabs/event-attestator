use crate::{
    eos_on_eth::eth::eos_tx_info::{EosOnEthEosTxInfo, EosOnEthEosTxInfos},
    safe_addresses::SAFE_EOS_ADDRESS_STR,
    state::EthState,
    traits::DatabaseInterface,
    types::Result,
};

create_safe_address_diversion_fxns_v2!(
    "EosOnEthEosTxInfo" => EthState => "eos" => SAFE_EOS_ADDRESS_STR.to_string() => String =>"token"
);
