use crate::{
    int_on_algo::algo::int_tx_info::{IntOnAlgoIntTxInfo, IntOnAlgoIntTxInfos},
    state::AlgoState,
    traits::TxInfo,
};

impl_safe_address_diversion_fxn_v2!("zero", AlgoState<D>, int_on_algo_int_tx_info);
impl_safe_address_diversion_fxn_v2!("vault", AlgoState<D>, int_on_algo_int_tx_info);
impl_safe_address_diversion_fxn_v2!("token", AlgoState<D>, int_on_algo_int_tx_info);
impl_safe_address_diversion_fxn_v2!("router", AlgoState<D>, int_on_algo_int_tx_info);
