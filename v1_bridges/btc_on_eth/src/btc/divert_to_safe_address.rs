use common::{safe_addresses::SAFE_ETH_ADDRESS, traits::DatabaseInterface, types::Result};
use common_btc::BtcState;
use ethereum_types::Address as EthAddress;

use crate::btc::eth_tx_info::{BtcOnEthEthTxInfo, BtcOnEthEthTxInfos};

create_safe_address_diversion_fxns_v2!(
    "BtcOnEthEthTxInfo" => BtcState => "eth" => *SAFE_ETH_ADDRESS => EthAddress => "token"
);
