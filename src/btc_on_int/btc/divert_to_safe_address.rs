use crate::{
    btc_on_int::btc::int_tx_info::{BtcOnIntIntTxInfo, BtcOnIntIntTxInfos},
    chains::btc::btc_state::BtcState,
    safe_addresses::SAFE_ETH_ADDRESS_HEX,
    traits::DatabaseInterface,
    types::Result,
};

create_safe_address_diversion_fxns!(
    "BtcOnIntIntTxInfo" => BtcState => "int" => SAFE_ETH_ADDRESS_HEX.to_string() => String => "token"
);
