use crate::{
    eos_on_int::int::eos_tx_info::{EosOnIntEosTxInfo, EosOnIntEosTxInfos},
    safe_addresses::SAFE_EOS_ADDRESS_STR,
    state::EthState,
    traits::DatabaseInterface,
    types::Result,
};

create_safe_address_diversion_fxns_v2!("EosOnIntEosTxInfo" => EthState => "eos" => SAFE_EOS_ADDRESS_STR.to_string() => String =>"token");
/*
use common::{state::BtcState, traits::TxInfo};

use crate::btc::{BtcOnIntIntTxInfo, BtcOnIntIntTxInfos};

impl_safe_address_diversion_fxn_v3!("zero", BtcState<D>, btc_on_int_int_tx_info);
impl_safe_address_diversion_fxn_v3!("token", BtcState<D>, btc_on_int_int_tx_info);
impl_safe_address_diversion_fxn_v3!("router", BtcState<D>, btc_on_int_int_tx_info);
*/
