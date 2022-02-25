use ethereum_types::Address as EthAddress;

use crate::{
    chains::eth::eth_state::EthState,
    constants::SAFE_ETH_ADDRESS,
    int_on_evm::evm::int_tx_info::{IntOnEvmIntTxInfo, IntOnEvmIntTxInfos},
    traits::DatabaseInterface,
    types::Result,
};

create_safe_address_diversion_fxns!(
    "IntOnEvmIntTxInfo" => EthState => "eth" => *SAFE_ETH_ADDRESS => EthAddress => "token", "vault"
);
