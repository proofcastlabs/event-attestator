use ethereum_types::Address as EthAddress;

use crate::{
    chains::eth::eth_state::EthState,
    erc20_on_int::int::eth_tx_info::{EthOnIntEthTxInfo, EthOnIntEthTxInfos},
    traits::DatabaseInterface,
    types::Result,
};

create_diversion_fxns!("EthOnIntEthTxInfo" => "Eth" => "erc20_on_int_eth_tx_infos" => "vault", "token");
