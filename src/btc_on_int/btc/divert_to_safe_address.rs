use ethereum_types::Address as EthAddress;

use crate::{
    btc_on_int::btc::int_tx_info::{BtcOnIntIntTxInfo, BtcOnIntIntTxInfos},
    chains::btc::btc_state::BtcState,
    constants::SAFE_ETH_ADDRESS,
    traits::DatabaseInterface,
    types::Result,
};

create_safe_address_diversion_fxns!(
    "BtcOnIntIntTxInfo" => BtcState => "int" => *SAFE_ETH_ADDRESS => EthAddress => "token"
);
