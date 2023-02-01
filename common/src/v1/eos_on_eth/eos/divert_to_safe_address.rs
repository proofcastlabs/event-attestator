use ethereum_types::Address as EthAddress;

use crate::{
    eos_on_eth::eos::eos_tx_info::{EosOnEthEosTxInfo, EosOnEthEosTxInfos},
    safe_addresses::SAFE_ETH_ADDRESS,
    state::EosState,
    traits::DatabaseInterface,
    types::Result,
};

create_safe_address_diversion_fxns!(
    "EosOnEthEosTxInfo" => EosState => "eth" => *SAFE_ETH_ADDRESS => EthAddress => "token"
);
