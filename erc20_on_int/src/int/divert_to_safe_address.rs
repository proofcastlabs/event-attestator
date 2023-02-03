use common::{state::EthState, traits::TxInfo};

use crate::int::eth_tx_info::{Erc20OnIntEthTxInfo, Erc20OnIntEthTxInfos};

impl_safe_address_diversion_fxn_v3!("zero", EthState<D>, erc20_on_int_eth_tx_info);
impl_safe_address_diversion_fxn_v3!("vault", EthState<D>, erc20_on_int_eth_tx_info);
impl_safe_address_diversion_fxn_v3!("token", EthState<D>, erc20_on_int_eth_tx_info);
impl_safe_address_diversion_fxn_v3!("router", EthState<D>, erc20_on_int_eth_tx_info);
