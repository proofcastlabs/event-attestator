use crate::{
    chains::eth::eth_state::EthState,
    constants::SAFE_ETH_ADDRESS_HEX,
    erc20_on_int::eth::int_tx_info::{Erc20OnIntIntTxInfo, Erc20OnIntIntTxInfos},
    traits::DatabaseInterface,
    types::Result,
};

create_safe_address_diversion_fxns!(
    "Erc20OnIntIntTxInfo" => EthState => "evm" => SAFE_ETH_ADDRESS_HEX.to_string() => String => "token"
);
