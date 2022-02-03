use ethereum_types::Address as EthAddress;

use crate::{
    chains::eos::eos_state::EosState,
    eos_on_eth::eos::eos_tx_info::{EosOnEthEosTxInfo, EosOnEthEosTxInfos},
    traits::DatabaseInterface,
    types::Result,
};

create_eth_safe_address_diversion_fxns!(
    "EosOnEthEosTxInfo" => "Eos" => "eos_on_eth_eos_tx_infos" => "token"
);
