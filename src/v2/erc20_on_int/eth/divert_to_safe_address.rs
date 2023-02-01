use crate::{
    chains::eth::eth_state::EthState,
    erc20_on_int::eth::int_tx_info::{Erc20OnIntIntTxInfo, Erc20OnIntIntTxInfos},
    traits::TxInfo,
};

impl_safe_address_diversion_fxn_v2!("zero", EthState<D>, erc20_on_int_int_tx_info);
impl_safe_address_diversion_fxn_v2!("vault", EthState<D>, erc20_on_int_int_tx_info);
impl_safe_address_diversion_fxn_v2!("token", EthState<D>, erc20_on_int_int_tx_info);
impl_safe_address_diversion_fxn_v2!("router", EthState<D>, erc20_on_int_int_tx_info);
