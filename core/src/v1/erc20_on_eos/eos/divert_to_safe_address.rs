use ethereum_types::Address as EthAddress;

use crate::{
    erc20_on_eos::eos::eth_tx_info::{Erc20OnEosEthTxInfo, Erc20OnEosEthTxInfos},
    safe_addresses::SAFE_ETH_ADDRESS,
    state::EosState,
    traits::DatabaseInterface,
    types::Result,
};

create_safe_address_diversion_fxns!(
    "Erc20OnEosEthTxInfo" => EosState => "eth" => *SAFE_ETH_ADDRESS => EthAddress => "token", "vault"
);
