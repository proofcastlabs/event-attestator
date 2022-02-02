use ethereum_types::Address as EthAddress;

use crate::{
    chains::eth::eth_state::EthState,
    erc20_on_evm::eth::evm_tx_info::{EthOnEvmEvmTxInfo, EthOnEvmEvmTxInfos},
    traits::DatabaseInterface,
    types::Result,
};

create_eth_safe_address_diversion_fxns!(
    "EthOnEvmEvmTxInfo" => "Eth" => "erc20_on_evm_evm_tx_infos" => "token"
);
