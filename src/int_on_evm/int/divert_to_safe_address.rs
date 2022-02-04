use ethereum_types::Address as EthAddress;

use crate::{
    chains::eth::eth_state::EthState,
    int_on_evm::int::evm_tx_info::{IntOnEvmEvmTxInfo, IntOnEvmEvmTxInfos},
    traits::DatabaseInterface,
    types::Result,
};

create_eth_safe_address_diversion_fxns!("IntOnEvmEvmTxInfo" => "Eth" => "token");
