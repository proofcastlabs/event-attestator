use crate::{
    chains::eth::eth_state::EthState,
    constants::SAFE_EOS_ADDRESS_STR,
    eos_on_eth::eth::eth_tx_info::{EosOnEthEthTxInfo, EosOnEthEthTxInfos},
    traits::DatabaseInterface,
    types::Result,
};

create_safe_address_diversion_fxns!(
    "EosOnEthEthTxInfo" => EthState => "eos" => SAFE_EOS_ADDRESS_STR.to_string() => String =>"token"
);
