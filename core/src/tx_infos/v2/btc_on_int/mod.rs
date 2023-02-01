mod btc_tx_info;
mod int_tx_info;

pub use self::{
    btc_tx_info::{BtcOnIntBtcTxInfo, BtcOnIntBtcTxInfos},
    int_tx_info::{BtcOnIntIntTxInfo, BtcOnIntIntTxInfos},
};
