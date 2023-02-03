use common::{state::AlgoState, traits::TxInfo};

use crate::algo::int_tx_info::{IntOnAlgoIntTxInfo, IntOnAlgoIntTxInfos};

impl_safe_address_diversion_fxn_v3!("zero", AlgoState<D>, int_on_algo_int_tx_info);
impl_safe_address_diversion_fxn_v3!("vault", AlgoState<D>, int_on_algo_int_tx_info);
impl_safe_address_diversion_fxn_v3!("token", AlgoState<D>, int_on_algo_int_tx_info);
impl_safe_address_diversion_fxn_v3!("router", AlgoState<D>, int_on_algo_int_tx_info);
