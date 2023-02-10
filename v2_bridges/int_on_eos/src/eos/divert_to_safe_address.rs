use common::{chains::eos::EosState, traits::TxInfo};

use crate::eos::int_tx_info::{IntOnEosIntTxInfo, IntOnEosIntTxInfos};

impl_safe_address_diversion_fxn_v3!("zero", EosState<D>, int_on_eos_int_tx_info);
impl_safe_address_diversion_fxn_v3!("vault", EosState<D>, int_on_eos_int_tx_info);
impl_safe_address_diversion_fxn_v3!("token", EosState<D>, int_on_eos_int_tx_info);
impl_safe_address_diversion_fxn_v3!("router", EosState<D>, int_on_eos_int_tx_info);
