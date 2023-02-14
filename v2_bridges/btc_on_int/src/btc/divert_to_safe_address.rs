use common::{chains::btc::BtcState, traits::TxInfo};

use crate::btc::{BtcOnIntIntTxInfo, BtcOnIntIntTxInfos};

impl_safe_address_diversion_fxn_v3!("zero", BtcState<D>, btc_on_int_int_tx_info);
impl_safe_address_diversion_fxn_v3!("token", BtcState<D>, btc_on_int_int_tx_info);
impl_safe_address_diversion_fxn_v3!("router", BtcState<D>, btc_on_int_int_tx_info);
