use crate::{
    chains::eth::eth_state::EthState,
    erc20_on_eos::eth::peg_in_info::{Erc20OnEosPegInInfo, Erc20OnEosPegInInfos},
    traits::DatabaseInterface,
    types::Result,
};

create_eos_safe_address_diversion_fxns!("Erc20OnEosPegInInfo" => "Eth" => "token");
