use ethereum_types::Address as EthAddress;

use crate::{
    chains::eth::eth_state::EthState,
    erc20_on_evm::evm::eth_tx_info::{Erc20OnEvmEthTxInfo, Erc20OnEvmEthTxInfos},
    traits::DatabaseInterface,
    types::Result,
};

create_eth_safe_address_diversion_fxns!("Erc20OnEvmEthTxInfo" => "Eth" => "token", "vault");
