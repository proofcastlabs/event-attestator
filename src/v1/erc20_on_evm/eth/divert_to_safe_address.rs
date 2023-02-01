use ethereum_types::Address as EthAddress;

use crate::{
    chains::eth::eth_state::EthState,
    erc20_on_evm::eth::evm_tx_info::{Erc20OnEvmEvmTxInfo, Erc20OnEvmEvmTxInfos},
    safe_addresses::SAFE_ETH_ADDRESS,
    traits::DatabaseInterface,
    types::Result,
};

create_safe_address_diversion_fxns!(
    "Erc20OnEvmEvmTxInfo" => EthState => "evm" => *SAFE_ETH_ADDRESS => EthAddress => "token"
);
