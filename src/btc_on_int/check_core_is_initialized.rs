use crate::{
    chains::{
        btc::{
            btc_database_utils::BtcDbUtils,
            btc_state::BtcState,
            core_initialization::check_btc_core_is_initialized::check_btc_core_is_initialized,
        },
        eth::{
            core_initialization::check_eth_core_is_initialized::check_eth_core_is_initialized,
            eth_database_utils::EthDbUtils,
            eth_state::EthState,
        },
    },
    traits::DatabaseInterface,
    types::Result,
};

pub fn check_core_is_initialized<D: DatabaseInterface>(
    eth_db_utils: &EthDbUtils<D>,
    btc_db_utils: &BtcDbUtils<D>,
) -> Result<()> {
    info!("✔ Checking core is initialized...");
    check_btc_core_is_initialized(btc_db_utils).and_then(|_| check_eth_core_is_initialized(eth_db_utils))
}

pub fn check_core_is_initialized_and_return_btc_state<D: DatabaseInterface>(state: BtcState<D>) -> Result<BtcState<D>> {
    check_core_is_initialized(&state.eth_db_utils, &state.btc_db_utils).and(Ok(state))
}

pub fn check_core_is_initialized_and_return_eth_state<D: DatabaseInterface>(state: EthState<D>) -> Result<EthState<D>> {
    check_core_is_initialized(&state.eth_db_utils, &state.btc_db_utils).and(Ok(state))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        chains::{
            btc::{btc_database_utils::BtcDbUtils, btc_test_utils::SAMPLE_TARGET_BTC_ADDRESS},
            eth::{
                eth_database_utils::{EthDbUtils, EthDbUtilsExt},
                eth_test_utils::get_sample_eth_address,
            },
        },
        test_utils::get_test_database,
    };

    #[test]
    fn should_err_if_core_not_initialized() {
        let db = get_test_database();
        let eth_db_utils = EthDbUtils::new(&db);
        let btc_db_utils = BtcDbUtils::new(&db);
        let result = check_core_is_initialized(&eth_db_utils, &btc_db_utils);
        assert!(result.is_err());
    }

    #[test]
    fn should_be_ok_if_core_initialized() {
        let db = get_test_database();
        let btc_db_utils = BtcDbUtils::new(&db);
        let eth_db_utils = EthDbUtils::new(&db);
        btc_db_utils.put_btc_address_in_db(SAMPLE_TARGET_BTC_ADDRESS).unwrap();
        eth_db_utils
            .put_public_eth_address_in_db(&get_sample_eth_address())
            .unwrap();
        let result = check_core_is_initialized(&eth_db_utils, &btc_db_utils);
        assert!(result.is_ok());
    }
}
