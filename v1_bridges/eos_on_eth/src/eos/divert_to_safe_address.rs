use common::{traits::DatabaseInterface, types::Result};
use common_eos::EosState;
use common_safe_addresses::SAFE_ETH_ADDRESS;
use ethereum_types::Address as EthAddress;

use crate::eos::{EosOnEthEthTxInfo, EosOnEthEthTxInfos};

create_safe_address_diversion_fxns_v2!(
    "EosOnEthEthTxInfo" => EosState => "eth" => *SAFE_ETH_ADDRESS => EthAddress => "token"
);
