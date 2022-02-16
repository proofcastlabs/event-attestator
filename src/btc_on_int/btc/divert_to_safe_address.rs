use ethereum_types::Address as EthAddress;

use crate::{
    btc_on_eth::btc::eth_tx_info::{BtcOnEthEthTxInfo, BtcOnEthEthTxInfos},
    chains::btc::btc_state::BtcState,
    constants::SAFE_ETH_ADDRESS,
    traits::DatabaseInterface,
    types::Result,
};

create_safe_address_diversion_fxns!(
    "BtcOnEthEthTxInfo" => BtcState => "eth" => *SAFE_ETH_ADDRESS => EthAddress => "token"
);
