use ethereum_types::Address as EthAddress;

use crate::{
    chains::eth::eth_state::EthState,
    erc20_on_evm::evm::eth_tx_info::{EthOnEvmEthTxInfo, EthOnEvmEthTxInfos},
    traits::DatabaseInterface,
    types::Result,
};

create_eth_safe_address_diversion_fxns!(
    "EthOnEvmEthTxInfo" => "Eth" => "erc20_on_evm_eth_tx_infos" => "token", "vault"
);
