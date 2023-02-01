use crate::{
    eos_on_int::eos::int_tx_info::{EosOnIntIntTxInfo, EosOnIntIntTxInfos},
    state::EosState,
    traits::TxInfo,
};

impl_safe_address_diversion_fxn_v2!("zero", EosState<D>, eos_on_int_int_tx_info);
impl_safe_address_diversion_fxn_v2!("token", EosState<D>, eos_on_int_int_tx_info);
impl_safe_address_diversion_fxn_v2!("router", EosState<D>, eos_on_int_int_tx_info);
