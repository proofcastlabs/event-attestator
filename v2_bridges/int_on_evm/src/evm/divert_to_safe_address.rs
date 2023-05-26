use common::traits::TxInfo;
use common_eth::EthState;

use crate::evm::int_tx_info::{IntOnEvmIntTxInfo, IntOnEvmIntTxInfos};

impl_safe_address_diversion_fxn_v3!("zero", EthState<D>, int_on_evm_int_tx_info);
impl_safe_address_diversion_fxn_v3!("vault", EthState<D>, int_on_evm_int_tx_info);
impl_safe_address_diversion_fxn_v3!("token", EthState<D>, int_on_evm_int_tx_info);
impl_safe_address_diversion_fxn_v3!("router", EthState<D>, int_on_evm_int_tx_info);
