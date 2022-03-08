use crate::{
    chains::eth::eth_state::EthState,
    constants::SAFE_EOS_ADDRESS_STR,
    erc20_on_eos::eth::peg_in_info::{Erc20OnEosPegInInfo, Erc20OnEosPegInInfos},
    traits::DatabaseInterface,
    types::Result,
};

create_safe_address_diversion_fxns!(
    "Erc20OnEosPegInInfo" => EthState => "eos" => SAFE_EOS_ADDRESS_STR.to_string() => String => "token"
);
