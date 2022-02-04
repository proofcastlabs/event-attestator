use ethereum_types::Address as EthAddress;

use crate::{
    chains::eth::eth_state::EthState,
    erc20_on_int::eth::int_tx_info::{Erc20OnIntIntTxInfo, Erc20OnIntIntTxInfos},
    traits::DatabaseInterface,
    types::Result,
};

create_eth_safe_address_diversion_fxns!("Erc20OnIntIntTxInfo" => "Eth" => "token");
