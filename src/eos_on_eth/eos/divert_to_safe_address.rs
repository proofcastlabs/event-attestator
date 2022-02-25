use ethereum_types::Address as EthAddress;

use crate::{
    chains::eos::eos_state::EosState,
    constants::SAFE_ETH_ADDRESS,
    eos_on_eth::eos::eos_tx_info::{EosOnEthEosTxInfo, EosOnEthEosTxInfos},
    traits::DatabaseInterface,
    types::Result,
};

create_safe_address_diversion_fxns!(
    "EosOnEthEosTxInfo" => EosState => "eth" => *SAFE_ETH_ADDRESS => EthAddress => "token"
);
