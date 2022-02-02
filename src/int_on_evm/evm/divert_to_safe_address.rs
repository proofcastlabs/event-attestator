use ethereum_types::Address as EthAddress;

use crate::{
    chains::eth::eth_state::EthState,
    int_on_evm::evm::int_tx_info::{IntOnEvmIntTxInfo, IntOnEvmIntTxInfos},
    traits::DatabaseInterface,
    types::Result,
};

create_eth_safe_address_diversion_fxns!(
    "IntOnEvmIntTxInfo" => "Eth" => "int_on_evm_int_tx_infos" => "token", "vault"
);
