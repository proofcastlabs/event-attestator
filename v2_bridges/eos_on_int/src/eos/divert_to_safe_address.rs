use common::traits::TxInfo;
use common_eos::EosState;

use crate::eos::int_tx_info::{EosOnIntIntTxInfo, EosOnIntIntTxInfos};

impl_safe_address_diversion_fxn_v3!("zero", EosState<D>, eos_on_int_int_tx_info);
impl_safe_address_diversion_fxn_v3!("token", EosState<D>, eos_on_int_int_tx_info);
impl_safe_address_diversion_fxn_v3!("router", EosState<D>, eos_on_int_int_tx_info);
