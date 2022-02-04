use ethereum_types::Address as EthAddress;

use crate::{
    btc_on_eth::btc::eth_tx_info::{BtcOnEthEthTxInfo, BtcOnEthEthTxInfos},
    chains::btc::btc_state::BtcState,
    create_eth_safe_address_diversion_fxns,
    traits::DatabaseInterface,
    types::Result,
};

create_eth_safe_address_diversion_fxns!("BtcOnEthEthTxInfo" => "Btc" => "token");
