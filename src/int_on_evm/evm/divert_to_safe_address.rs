use crate::{
    chains::eth::eth_state::EthState,
    int_on_evm::evm::int_tx_info::{IntOnEvmIntTxInfo, IntOnEvmIntTxInfos},
    safe_addresses::SAFE_ETH_ADDRESS_HEX,
    traits::DatabaseInterface,
    types::Result,
};

create_safe_address_diversion_fxns!(
    "IntOnEvmIntTxInfo" => EthState => "eth" => SAFE_ETH_ADDRESS_HEX.to_string() => String => "token", "vault"
);
