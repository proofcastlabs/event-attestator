use crate::{
    int_on_evm::evm::int_tx_info::{IntOnEvmIntTxInfo, IntOnEvmIntTxInfos},
    state::EthState,
    traits::TxInfo,
};

impl_safe_address_diversion_fxn_v3!("zero", EthState<D>, int_on_evm_int_tx_info);
impl_safe_address_diversion_fxn_v3!("vault", EthState<D>, int_on_evm_int_tx_info);
impl_safe_address_diversion_fxn_v3!("token", EthState<D>, int_on_evm_int_tx_info);
impl_safe_address_diversion_fxn_v3!("router", EthState<D>, int_on_evm_int_tx_info);
