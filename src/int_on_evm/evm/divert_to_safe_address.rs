use crate::{
    chains::eth::eth_state::EthState,
    constants::SAFE_ETH_ADDRESS_HEX,
    int_on_evm::evm::int_tx_info::{IntOnEvmIntTxInfo, IntOnEvmIntTxInfos},
    traits::DatabaseInterface,
    types::Result,
};

create_safe_address_diversion_fxns!(
    "IntOnEvmIntTxInfo" => EthState => "eth" => SAFE_ETH_ADDRESS_HEX.to_string() => String => "token", "vault"
);
