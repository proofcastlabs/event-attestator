use ethereum_types::Address as EthAddress;

use crate::{
    chains::eth::eth_state::EthState,
    erc20_on_int::int::eth_tx_info::{Erc20OnIntEthTxInfo, Erc20OnIntEthTxInfos},
    safe_addresses::SAFE_ETH_ADDRESS,
    traits::DatabaseInterface,
    types::Result,
};

create_safe_address_diversion_fxns!(
    "Erc20OnIntEthTxInfo" => EthState => "eth" => *SAFE_ETH_ADDRESS => EthAddress => "token", "vault"
);
