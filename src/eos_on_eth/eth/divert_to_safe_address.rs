use crate::{
    chains::eth::eth_state::EthState,
    eos_on_eth::eth::eth_tx_info::{EosOnEthEthTxInfo, EosOnEthEthTxInfos},
    traits::DatabaseInterface,
    types::Result,
};

create_eos_safe_address_diversion_fxns!("EosOnEthEthTxInfo" => "Eth" => "token");
