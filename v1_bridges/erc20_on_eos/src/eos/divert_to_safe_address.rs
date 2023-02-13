use common::{safe_addresses::SAFE_ETH_ADDRESS, traits::DatabaseInterface, types::Result};
use common_eos::EosState;
use ethereum_types::Address as EthAddress;

use crate::eos::eth_tx_info::{Erc20OnEosEthTxInfo, Erc20OnEosEthTxInfos};

create_safe_address_diversion_fxns_v2!(
    "Erc20OnEosEthTxInfo" => EosState => "eth" => *SAFE_ETH_ADDRESS => EthAddress => "token", "vault"
);
