use ethereum_types::Address as EthAddress;

use crate::{
    chains::eth::eth_state::EthState,
    erc20_on_int::eth::int_tx_info::{EthOnIntIntTxInfo, EthOnIntIntTxInfos},
    traits::DatabaseInterface,
    types::Result,
};

create_eth_safe_address_diversion_fxns!(
    "EthOnIntIntTxInfo" => "Eth" => "erc20_on_int_int_tx_infos" => "token"
);
