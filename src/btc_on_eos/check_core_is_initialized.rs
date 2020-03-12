use crate::btc_on_eos::{
    types::Result,
    errors::AppError,
    traits::DatabaseInterface,
    eos::{
        eos_state::EosState,
        initialize_eos::is_eos_core_initialized::is_eos_core_initialized,
    },
    /*
    btc::{
        btc_state::BtcState,
        initialize_btc::is_btc_initialized::is_btc_core_initialized,
    },
    */
};

pub fn check_core_is_initialized<D>(
    db: &D
) -> Result<()>
    where D: DatabaseInterface
{
    info!("✔ Checking core is initialized...");
    match is_eos_core_initialized(db) {
        false => Err(AppError::Custom(
            "✘ EOS side of core not initialized!".to_string()
        )),
        true => Ok(())
            /*
        {
            match is_btc_core_initialized(db) {
                false => Err(AppError::Custom(
                    "✘ BTC side of core not initialized!".to_string()
                )),
                true => Ok(())
            }
        }
        */
    }
}

// TODO/FIXME Make generic
pub fn check_core_is_initialized_and_return_eos_state<D>(
    state: EosState<D>,
) -> Result<EosState<D>>
    where D: DatabaseInterface
{
    check_core_is_initialized(&state.db)
        .map(|_| state)
}

/*
pub fn check_core_is_initialized_and_return_btc_state<D>(
    state: BtcState<D>,
) -> Result<BtcState<D>>
    where D: DatabaseInterface
{
    check_core_is_initialized(&state.db)
        .map(|_| state)
}
*/

/* TODO reinstate
#[cfg(test)]
mod tests {
    use super::*;
    use crate::btc_on_eth::{
        test_utils::get_test_database,
        btc::{
            btc_test_utils::SAMPLE_TARGET_BTC_ADDRESS,
            btc_database_utils::put_btc_address_in_db,
        },
        eth::{
            eth_database_utils::put_public_eth_address_in_db,
            eth_test_utils::{
                get_sample_eth_address,
                get_valid_eth_state,
            },
        },
    };

    #[test]
    fn should_return_false_if_core_not_initialized() {
        if let Ok(_) = check_core_is_initialized(&get_test_database()) {
            panic!("Enc should be initialized!");
        }
    }

    #[test]
    fn should_return_true_if_core_initialized() {
        let db = get_test_database();
        if let Err(e) = put_btc_address_in_db(
            &db,
            &SAMPLE_TARGET_BTC_ADDRESS.to_string(),
        ) {
            panic!("Error putting pk in db: {}", e);
        };
        if let Err(e) = put_public_eth_address_in_db(
            &db,
            &get_sample_eth_address(),
        ) {
            panic!("Error putting pk in db: {}", e);
        };
        if let Err(e) = check_core_is_initialized(&db) {
            panic!("Error when enc should be initted: {}", e);
        };
    }

    #[test]
    fn should_error_if_btc_core_not_initialized() {
        let db = get_test_database();
        let expected_error = "✘ BTC side of core not initialized!"
            .to_string();
        if let Err(e) = put_public_eth_address_in_db(
            &db,
            &get_sample_eth_address(),
        ) {
            panic!("Error putting pk in db: {}", e);
        };
        assert!(!is_btc_core_initialized(&db));
        match check_core_is_initialized(&db) {
            Ok(_) => {
                panic!("Enc should not be initialized!");
            }
            Err(AppError::Custom(e)) => {
                assert!(e == expected_error);
            }
            Err(e) => {
                panic!("Wrong err recieved: {}", e);
            }
        }
    }

    #[test]
    fn should_error_if_eth_core_not_initialized() {
        let db = get_test_database();
        let expected_error = "✘ ETH side of core not initialized!"
            .to_string();
        if let Err(e) = put_btc_address_in_db(
            &db,
            &SAMPLE_TARGET_BTC_ADDRESS.to_string(),
        ) {
            panic!("Error putting pk in db: {}", e);
        };
        assert!(
            !is_eth_core_initialized(&db)
        );
        match check_core_is_initialized(&db) {
            Ok(_) => {
                panic!("Enc should not be initialized!");
            }
            Err(AppError::Custom(e)) => {
                assert!(e == expected_error);
            }
            Err(e) => {
                panic!("Wrong err recieved: {}", e);
            }
        }
    }

    #[test]
    fn should_check_core_initialized_and_return_arg() {
        let state = get_valid_eth_state()
            .unwrap();
        if let Err(e) = put_btc_address_in_db(
            &state.db,
            &SAMPLE_TARGET_BTC_ADDRESS.to_string(),
        ) {
            panic!("Error putting pk in db: {}", e);
        };
        if let Err(e) = put_public_eth_address_in_db(
            &state.db,
            &get_sample_eth_address(),
        ) {
            panic!("Error putting pk in db: {}", e);
        };
        if let Err(e)  = check_core_is_initialized_and_return_eth_state(
            state
        ) {
            panic!("Error when enc should be initted: {}", e);
        }
    }
}
*/
