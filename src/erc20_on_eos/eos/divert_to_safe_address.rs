use ethereum_types::Address as EthAddress;

use crate::{
    chains::eos::eos_state::EosState,
    erc20_on_eos::eos::redeem_info::{Erc20OnEosRedeemInfo, Erc20OnEosRedeemInfos},
    traits::DatabaseInterface,
    types::Result,
};

create_eth_safe_address_diversion_fxns!(
    "Erc20OnEosRedeemInfo" => "Eos" => "erc20_on_eos_redeem_infos" => "token", "vault"
);
