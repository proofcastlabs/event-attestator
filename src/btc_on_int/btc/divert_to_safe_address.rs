use crate::{
    btc_on_int::btc::int_tx_info::{BtcOnIntIntTxInfo, BtcOnIntIntTxInfos},
    chains::btc::btc_state::BtcState,
    traits::TxInfo,
};

impl_safe_address_diversion_fxn_v2!("zero", BtcState<D>, btc_on_int_int_tx_info);
impl_safe_address_diversion_fxn_v2!("token", BtcState<D>, btc_on_int_int_tx_info);
impl_safe_address_diversion_fxn_v2!("router", BtcState<D>, btc_on_int_int_tx_info);
