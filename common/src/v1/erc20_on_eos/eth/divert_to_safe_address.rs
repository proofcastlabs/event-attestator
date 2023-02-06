use crate::{
    erc20_on_eos::eth::eos_tx_info::{Erc20OnEosEosTxInfo, Erc20OnEosEosTxInfos},
    safe_addresses::SAFE_EOS_ADDRESS_STR,
    state::EthState,
    traits::DatabaseInterface,
    types::Result,
};

create_safe_address_diversion_fxns_v2!(
    "Erc20OnEosEosTxInfo" => EthState => "eos" => SAFE_EOS_ADDRESS_STR.to_string() => String => "token"
);
