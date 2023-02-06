use ethereum_types::Address as EthAddress;

use crate::{
    eos_on_eth::eos::{EosOnEthEthTxInfo, EosOnEthEthTxInfos},
    safe_addresses::SAFE_ETH_ADDRESS,
    state::EosState,
    traits::DatabaseInterface,
    types::Result,
};

create_safe_address_diversion_fxns_v2!(
    "EosOnEthEthTxInfo" => EosState => "eth" => *SAFE_ETH_ADDRESS => EthAddress => "token"
);
