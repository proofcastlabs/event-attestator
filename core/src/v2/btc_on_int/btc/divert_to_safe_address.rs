use crate::{
    state::BtcState,
    traits::TxInfo,
    tx_infos::{BtcOnIntIntTxInfo, BtcOnIntIntTxInfos},
};

impl_safe_address_diversion_fxn_v2!("zero", BtcState<D>, btc_on_int_int_tx_info);
impl_safe_address_diversion_fxn_v2!("token", BtcState<D>, btc_on_int_int_tx_info);
impl_safe_address_diversion_fxn_v2!("router", BtcState<D>, btc_on_int_int_tx_info);
