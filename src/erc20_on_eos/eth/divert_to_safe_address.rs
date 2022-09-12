use crate::{
    chains::eth::eth_state::EthState,
    erc20_on_eos::eth::eos_tx_info::{Erc20OnEosEosTxInfo, Erc20OnEosEosTxInfos},
    safe_addresses::SAFE_EOS_ADDRESS_STR,
    traits::DatabaseInterface,
    types::Result,
};

create_safe_address_diversion_fxns!(
    "Erc20OnEosEosTxInfo" => EthState => "eos" => SAFE_EOS_ADDRESS_STR.to_string() => String => "token"
);
