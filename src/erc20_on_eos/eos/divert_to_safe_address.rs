use ethereum_types::Address as EthAddress;

use crate::{
    chains::eos::eos_state::EosState,
    erc20_on_eos::eos::redeem_info::{Erc20OnEosRedeemInfo, Erc20OnEosRedeemInfos},
    safe_addresses::SAFE_ETH_ADDRESS,
    traits::DatabaseInterface,
    types::Result,
};

create_safe_address_diversion_fxns!(
    "Erc20OnEosRedeemInfo" => EosState => "eth" => *SAFE_ETH_ADDRESS => EthAddress => "token", "vault"
);
