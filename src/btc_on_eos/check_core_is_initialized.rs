use crate::{
    chains::{
        btc::{
            btc_database_utils::BtcDbUtils,
            btc_state::BtcState,
            core_initialization::check_btc_core_is_initialized::check_btc_core_is_initialized,
        },
        eos::{
            core_initialization::check_eos_core_is_initialized::check_eos_core_is_initialized,
            eos_database_utils::EosDbUtils,
            eos_state::EosState,
        },
    },
    traits::DatabaseInterface,
    types::Result,
};

pub(in crate::btc_on_eos) fn check_core_is_initialized<D: DatabaseInterface>(
    btc_db_utils: &BtcDbUtils<D>,
    eos_db_utils: &EosDbUtils<D>,
) -> Result<()> {
    check_btc_core_is_initialized(btc_db_utils).and_then(|_| check_eos_core_is_initialized(eos_db_utils))
}

pub(in crate::btc_on_eos) fn check_core_is_initialized_and_return_eos_state<D: DatabaseInterface>(
    state: EosState<D>,
) -> Result<EosState<D>> {
    check_core_is_initialized(&state.btc_db_utils, &state.eos_db_utils).and(Ok(state))
}

pub(in crate::btc_on_eos) fn check_core_is_initialized_and_return_btc_state<D: DatabaseInterface>(
    state: BtcState<D>,
) -> Result<BtcState<D>> {
    check_core_is_initialized(&state.btc_db_utils, &state.eos_db_utils).and(Ok(state))
}
