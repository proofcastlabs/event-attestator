use ethereum_types::Address as EthAddress;

use crate::{
    btc_on_eth::btc::eth_tx_info::{BtcOnEthEthTxInfo, BtcOnEthEthTxInfos},
    safe_addresses::SAFE_ETH_ADDRESS,
    state::BtcState,
    traits::DatabaseInterface,
    types::Result,
};

create_safe_address_diversion_fxns_v2!(
    "BtcOnEthEthTxInfo" => BtcState => "eth" => *SAFE_ETH_ADDRESS => EthAddress => "token"
);
