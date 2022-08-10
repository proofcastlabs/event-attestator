pub(crate) mod debug_gas_price_setters;
pub(crate) mod debug_nonce_setters;

pub use crate::chains::eth::eth_debug_functions::{
    debug_gas_price_setters::{debug_set_eth_gas_price, debug_set_evm_gas_price},
    debug_nonce_setters::{
        check_custom_nonce,
        debug_set_eth_account_nonce,
        debug_set_eth_any_sender_nonce,
        debug_set_evm_account_nonce,
        debug_set_evm_any_sender_nonce,
    },
};
